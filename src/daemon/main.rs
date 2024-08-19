use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::net::{SocketAddr, TcpListener, UdpSocket};
use std::sync::{Arc, Mutex};
use std::thread::{self, sleep};
use std::time::{Instant, Duration};

// Goals of this test program:
// [X] setup a UDP stream
// [X] send a raw audio file to the client
// [X] allow clients to subscribe to streams via tcp
// [X] stop stream when now subscribers
// [ ] test on pi
// [ ] access shm object on pi

fn transmit_audio(
    self_addr: Arc<Mutex<Option<SocketAddr>>>, 
    dest_addr: Arc<Mutex<Vec<SocketAddr>>>,
    stop_flag: Arc<Mutex<bool>>,
) -> std::io::Result<()>
{
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_broadcast(false)?;

    // socket.set_nonblocking(true).unwrap();
    let local_address = socket.local_addr()?;
    {
        let mut address = self_addr.lock().unwrap();
        *address = Some(local_address);
    }
    println!("Server listening on {:?}", local_address);
    let mut buffer = [0; 1496];

    let now = Instant::now();
    let mut audio_file = File::open("resources/hello_world.txt")?;
    let mut packet_index : u16 = 0;
    let mut stop = *stop_flag.lock().unwrap();
    while !stop {
        //check if any udp sockets are subscribed to audio stream.
        let dest_list = {(*dest_addr.lock().unwrap()).clone()};
        if !(*dest_list).is_empty(){
            // println!("sending packet");
            // Sending_packet
            // break aduio into transmittable chunk 
            let read_count = audio_file.read(& mut buffer[4..])?;
            buffer[0..2].copy_from_slice(&packet_index.to_be_bytes());
            buffer[2..4].copy_from_slice(&(read_count as u16).to_be_bytes());

            //send to all subscribed upd addresses
            for dest_addr in dest_list.iter(){
                socket.send_to(&buffer[..(4 + read_count)], dest_addr)?;
            }
            
            //increment packet index
            packet_index += 1;

            //check if file is completely sent
            if read_count == 0 {
                let elapsed = now.elapsed();
                println!("Reached end of file in {:.2?}", elapsed);
                break;
            }
        } else {
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
