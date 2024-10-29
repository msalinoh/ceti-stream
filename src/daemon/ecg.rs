use std::mem::{size_of};
use std::net::{SocketAddr, UdpSocket};
use std::ptr::{self};
use std::sync::{Arc, Mutex};
use std::thread::{sleep};
use std::time::{Duration};
use libc::{ftruncate, mmap, sem_open, sem_wait, shm_open};
use libc::{O_RDWR, S_IRUSR, PROT_READ, MAP_SHARED};
use libc::{c_char, c_int, c_long, c_longlong, off_t, sem_t};

const ECG_SHM_NAME: *const c_char = b"/ecg_shm\0".as_ptr() as *const c_char;
const ECG_SAMPLE_SEM_NAME: *const c_char = b"/ecg_sample_sem\0".as_ptr() as *const c_char;
const ECG_BLOCK_SEM_NAME: *const c_char = b"/ecg_page_sem\0".as_ptr() as *const c_char;

const ECG_NUM_BUFFER: usize = 2;
const ECG_BUFFER_LENGTH: usize = 1000;


#[repr(C)]
struct CetiEcgBuffer {
    page: c_int,
    sample: c_int,
    lod_enabled: c_int,
    sys_time_us: [c_longlong; ECG_BUFFER_LENGTH * ECG_NUM_BUFFER],
    rtc_time_s: [c_int; ECG_BUFFER_LENGTH * ECG_NUM_BUFFER],
    ecg_readings: [c_long; ECG_BUFFER_LENGTH * ECG_NUM_BUFFER],
    leads_off_readings_p: [c_int; ECG_BUFFER_LENGTH * ECG_NUM_BUFFER],
    leads_off_readings_n: [c_int; ECG_BUFFER_LENGTH * ECG_NUM_BUFFER],
    sample_indexes: [c_longlong; ECG_BUFFER_LENGTH * ECG_NUM_BUFFER],
}

// reformatted data
#[repr(C)]
struct CetiEcgSample {
    sys_time_us: c_longlong,
    rtc_time_s: c_int,
    ecg_readings: c_long,
    leads_off_readings_p: c_int,
    leads_off_readings_n: c_int,
    sample_indexes: c_longlong,
}

pub fn tx_thread(
    dest_addr: Arc<Mutex<Vec<SocketAddr>>>,
    stop_flag: Arc<Mutex<bool>>,
) -> std::io::Result<()>
{
    //open shared memory object
    let ecg_addr = unsafe {
        let null = ptr::null_mut();
        let fd = shm_open(ECG_SHM_NAME, O_RDWR, S_IRUSR);
        let _res = ftruncate(fd, size_of::<CetiEcgBuffer>() as off_t);
        let addr = mmap(null, size_of::<CetiEcgBuffer>(), PROT_READ, MAP_SHARED, fd, 0);
        addr as *const CetiEcgBuffer
    };

    // open ecg sample semaphore
    let ecg_sem : *mut sem_t= unsafe { sem_open(ECG_SAMPLE_SEM_NAME, O_RDWR, S_IRUSR, 0) };

    //create Udp socket
    let socket = UdpSocket::bind("0.0.0.0:0")?;

    let local_address = socket.local_addr()?;
    println!("Server listening on {:?}", local_address);

    let mut stop = *stop_flag.lock().unwrap();
    let mut paused = true; // flag for if there are no subscribers
    let mut read_offset = 0;
    let mut write_offset = 0;

    while !stop {
        //check if any udp sockets are subscribed to audio stream.
        let dest_list = {(*dest_addr.lock().unwrap()).clone()};
        if !(*dest_list).is_empty() {
            // there is someone subscribed to the udp stream
            if paused { //unpause the udp stream

                // wait for new ecg sample to be posted
                // Note: This may wait forever better to include timeout
                unsafe{let _res = sem_wait(ecg_sem);};
                let page = unsafe {(*ecg_addr).page as usize};
                let sample = unsafe {(*ecg_addr).sample as usize};
                write_offset = page * ECG_BUFFER_LENGTH + sample;   
                read_offset = write_offset;

                // wait for new data
                while read_offset == write_offset {
                    unsafe{let _res = sem_wait(ecg_sem);}; //get more data
                    //wait for more data if not enough in buffer
                    let page = unsafe {(*ecg_addr).page as usize};
                    let sample = unsafe {(*ecg_addr).sample as usize};
                    write_offset = page * ECG_BUFFER_LENGTH + sample;
                }
                println!("Offsets set to {:} and {:} [{:}][{:}]",read_offset, write_offset, page, sample);
                paused = false;
            }

            let sample_count = if (read_offset < write_offset) {
                write_offset - read_offset
            } else {
                write_offset + (ECG_BUFFER_LENGTH * ECG_NUM_BUFFER) - read_offset
            };

            
            
            while read_offset != write_offset {
                read_offset = (read_offset) + 1 % (ECG_BUFFER_LENGTH * ECG_NUM_BUFFER);

            }


            //send to all subscribed upd addresses
            // for dest_addr in dest_list.iter(){
            //     socket.send_to(&buffer, dest_addr)?;
            // }

        } else {
            // there is noone subscribed to the udp stream
            if !paused {
                paused = true;
            }
            println!("Waiting on subscribers");
            sleep(Duration::from_secs(2));
        }
        //update stop flag
        stop = *stop_flag.lock().unwrap();
    }

    println!("ECG Streaming thread has been stopped");
    return Ok(())
}