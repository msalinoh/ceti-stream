use std::env;
use std::io::prelude::*;
use std::mem::{size_of};
use std::net::{SocketAddr, TcpListener, UdpSocket};
use std::ptr::{self, read};
use std::sync::{Arc, Mutex};
use std::thread::{self, sleep};
use std::time::{Duration};
use libc::{ftruncate, mmap, shm_open, close, sem_open, sem_wait};
use libc::{O_RDWR, S_IRUSR, PROT_READ, MAP_SHARED};
use libc::{c_char, c_int, off_t, sem_t};

// Goals of this test program:
// [X] setup a UDP stream
// [X] send a raw audio file to the client
// [X] allow clients to subscribe to streams via tcp
// [X] stop stream when now subscribers
// [X] test on pi
// [X] access shm object on pi
// [ ] repeat for every sensor
const AUDIO_SHM_NAME: *const c_char = b"/audio_shm\0".as_ptr() as *const c_char; 
const AUDIO_CHANNELS: usize = 3;
//multiple of AUDIO_LCM_BYTES closest to 75 seconds @ 16-bit, 96kSPS, (14401536)
const AUDIO_BUFFER_SIZE_BYTES_PER_CHANNEL: usize = 14401536;
const AUDIO_BUFFER_SIZE_BYTES: usize = AUDIO_CHANNELS*AUDIO_BUFFER_SIZE_BYTES_PER_CHANNEL;
const AUDIO_BUFFER_BLOCK_SIZE_BYTES: usize = 512*32;

const AUDIO_BLOCK_SEM_NAME: *const c_char = b"/audio_block_sem\0".as_ptr() as *const c_char;
#[repr(C)]
struct CetiAudioBuffer{
    pub page: c_int,
    pub block: c_int,
    pub data: [u8; 2*AUDIO_BUFFER_SIZE_BYTES],
}

fn transmit_audio(
    self_addr: Arc<Mutex<Option<SocketAddr>>>, 
    dest_addr: Arc<Mutex<Vec<SocketAddr>>>,
    stop_flag: Arc<Mutex<bool>>,
) -> std::io::Result<()>
{
    //open shared memory object
    let (audio_fd, audio_addr) = unsafe {
        let null = ptr::null_mut();
        let fd = shm_open(AUDIO_SHM_NAME, O_RDWR, S_IRUSR);
        let _res = ftruncate(fd, size_of::<CetiAudioBuffer>() as off_t);
        let addr = mmap(null, size_of::<CetiAudioBuffer>(), PROT_READ, MAP_SHARED, fd, 0);
        (fd, addr as *mut CetiAudioBuffer)
    };
    
    //open audio semaphore
    let audio_sem : *mut sem_t= unsafe { sem_open(AUDIO_BLOCK_SEM_NAME, O_RDWR, S_IRUSR, 0) };
    
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_broadcast(false)?;

    // socket.set_nonblocking(true).unwrap();
    let local_address = socket.local_addr()?;
    {
        let mut address = self_addr.lock().unwrap();
        *address = Some(local_address);
    }
    println!("Server listening on {:?}", local_address);
    let mut buffer = [0; 1498];

    let mut packet_index : u16 = 0;
    let mut stop = *stop_flag.lock().unwrap();
    println!("Accessing semaphore",);
    let mut paused = true;
    let mut read_offset = 0;
    let mut write_offset = 0;
    while !stop {
        //check if any udp sockets are subscribed to audio stream.
        let dest_list = {(*dest_addr.lock().unwrap()).clone()};
        if !(*dest_list).is_empty(){
            if paused {
                unsafe{let _res = sem_wait(audio_sem);};
                let page = unsafe { (*audio_addr).page as usize};
                let block = unsafe {(*audio_addr).block as usize};
                write_offset = page * AUDIO_BUFFER_SIZE_BYTES + block*AUDIO_BUFFER_BLOCK_SIZE_BYTES;
                //align to full sample
                read_offset = (write_offset + 5)/6;
                read_offset *= 6;
                assert_eq!(read_offset % 6, 0);
                
                //wait for next block
                let old_write = write_offset;
                while old_write == write_offset {
                    unsafe{let _res = sem_wait(audio_sem);}; //get more data
                    //wait for more data in not enough in buffer
                    let page = unsafe { (*audio_addr).page as usize};
                    let block = unsafe {(*audio_addr).block as usize};
                    write_offset = page * AUDIO_BUFFER_SIZE_BYTES + block*AUDIO_BUFFER_BLOCK_SIZE_BYTES;
                }

                println!("Offsets set to {:} and {:} [{:}][{:}]",read_offset, write_offset, page, block);
                paused = false;
            }
            let mut remaining_bytes = 1494;
            //check if we need to read in more bytes
            if ( (read_offset == write_offset)
            || (read_offset < write_offset) && ((write_offset - read_offset) < remaining_bytes))
            || ((2*AUDIO_BUFFER_SIZE_BYTES - read_offset + write_offset) < remaining_bytes) 
            {
                // println!("Grabbing audio...",);
                let old_write = write_offset;
                while old_write == write_offset {
                    unsafe{let _res = sem_wait(audio_sem);}; //get more data
                    //wait for more data in not enough in buffer
                    let page = unsafe { (*audio_addr).page as usize};
                    let block = unsafe {(*audio_addr).block as usize};
                    write_offset = page * AUDIO_BUFFER_SIZE_BYTES + block*AUDIO_BUFFER_BLOCK_SIZE_BYTES;
                    if old_write != write_offset {
                        println!("Offsets set to {:} and {:} [{:}][{:}]",read_offset, write_offset, page, block);
                    }
                }
                continue;
            }
            
            buffer[0..2].copy_from_slice(&packet_index.to_be_bytes());
            buffer[2..4].copy_from_slice(&(1494 as u16).to_be_bytes());

            //copy audio into buffer
            //check if wrapped
            if (write_offset < read_offset) && (2*AUDIO_BUFFER_SIZE_BYTES - read_offset) < remaining_bytes {
                buffer[4..4+(2*AUDIO_BUFFER_SIZE_BYTES - read_offset)].copy_from_slice(unsafe{&(*audio_addr).data[read_offset..2*AUDIO_BUFFER_SIZE_BYTES]});
                remaining_bytes -= 2*AUDIO_BUFFER_SIZE_BYTES - read_offset;
                read_offset = 0;
            }
            buffer[4+(1494-remaining_bytes)..].copy_from_slice(unsafe{&(*audio_addr).data[read_offset..(read_offset + remaining_bytes)]});
            read_offset = (read_offset + remaining_bytes) % (2*AUDIO_BUFFER_SIZE_BYTES);
            
            //send to all subscribed upd addresses
            for dest_addr in dest_list.iter(){
                socket.send_to(&buffer, dest_addr)?;
            }
            
            //increment packet index
            packet_index += 1;

        } else {
            if !paused {
                paused = true;
            }
            println!("Waiting on subscribers");
            sleep(Duration::from_secs(2));
        }
        //update stop flag
        stop = *stop_flag.lock().unwrap();
    }
    println!("Audio Streaming thread has been stopped");
    return Ok(())
}

fn main() -> std::io::Result<()>{
    let audio_address : Arc<Mutex<Option<SocketAddr>>> = Arc::new(Mutex::new(None));
    let audio_sub_address : Arc<Mutex<Vec<SocketAddr>>> = Arc::new(Mutex::new(Vec::new()));
    let stop_flag : Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    
    let path = env::current_dir()?;
    println!("The current directory is {}", path.display());

    let listener = TcpListener::bind("0.0.0.0:8080")?;
    println!("Server listening on {:?}", listener.local_addr().unwrap());
    let a_addr = audio_address.clone();
    let a_sub_addr = audio_sub_address.clone();
    let a_stop_flag = stop_flag.clone();

    // start audio udp streaming thread
    let threads = thread::spawn(move || transmit_audio(a_addr, a_sub_addr, a_stop_flag));
    
    match listener.accept() {
        Ok((mut socket, source)) => {
            while !*stop_flag.lock().unwrap() {
                let mut rx_buffer = [0u8; 1500];
                let size = socket.read(& mut rx_buffer)?;
                println!("Rx'ed {:?}", String::from_utf8_lossy(&rx_buffer[..size]));

                if rx_buffer.starts_with(b"audio:") { //subscribe to audio
                    let udp_socket_port = u16::from_be_bytes([rx_buffer[6], rx_buffer[7]]);
                    let udp_socket_address = SocketAddr::from((source.ip(), udp_socket_port));
                    println!("Adding {:?} to subscriber list", udp_socket_address);
                    audio_sub_address.lock().unwrap().push(udp_socket_address);
                } else if rx_buffer.starts_with(b"unaudio:"){ //unsubscride to audio
                    let udp_socket_port = u16::from_be_bytes([rx_buffer[8], rx_buffer[9]]);
                    let udp_socket_address = SocketAddr::from((source.ip(), udp_socket_port));
                    println!("Removing {:?} from subscriber list", udp_socket_address);
                    audio_sub_address.lock().unwrap().retain(|&addr| addr != udp_socket_address);
                } else if rx_buffer.starts_with(b"stop") { //unsubscribe
                    println!("Received stop command");
                    *stop_flag.lock().unwrap() = true;
                } else {
                    println!("Receved \"{:?}\" from {:?}", String::from_utf8_lossy(&rx_buffer[..size]), source);
                }
            };
        },
        Err(e) => println!("couldn't get client: {e:?}"),
        
    }

    println!("Waiting on threads to finish!");
    threads.join().expect("thread panicked")?;
    println!("Goodbye!");
   
    return Ok(())
}
