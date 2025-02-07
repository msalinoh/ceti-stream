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

const UDP_PACKET_SIZE_MAX: usize = 1500;

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
    let ecg_addr: *const CetiEcgBuffer = unsafe {
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
                paused = false;
                break;
            }

            // wait for new data
            while read_offset == write_offset {
                unsafe{let _res = sem_wait(ecg_sem);}; //get more data
                //wait for more data if not enough in buffer
                let page = unsafe {(*ecg_addr).page as usize};
                let sample = unsafe {(*ecg_addr).sample as usize};
                write_offset = page * ECG_BUFFER_LENGTH + sample;
                println!("Offsets set to {:} and {:} [{:}][{:}]",read_offset, write_offset, page, sample);
            }
            
            //calculate number of new samples in buffer 
            let mut sample_count = if read_offset < write_offset {
                write_offset - read_offset
            } else {
                write_offset + (ECG_BUFFER_LENGTH * ECG_NUM_BUFFER) - read_offset
            };

            // transform ecg buffers into samples
            let mut samples : Vec<CetiEcgSample> = Vec::with_capacity(sample_count); 
            while read_offset != write_offset {
                let current_sample = CetiEcgSample{
                    sys_time_us: unsafe{(*ecg_addr).sys_time_us[read_offset]},
                    rtc_time_s: unsafe{(*ecg_addr).rtc_time_s[read_offset]},
                    ecg_readings: unsafe{(*ecg_addr).ecg_readings[read_offset]},
                    leads_off_readings_p: unsafe{(*ecg_addr).leads_off_readings_p[read_offset]},
                    leads_off_readings_n: unsafe{(*ecg_addr).leads_off_readings_n[read_offset]},
                    sample_indexes: unsafe{(*ecg_addr).sample_indexes[read_offset]},
                };
                samples.push(current_sample);
                read_offset = (read_offset) + 1 % (ECG_BUFFER_LENGTH * ECG_NUM_BUFFER);
            }

            // generate byte packages            
            const SAMPLES_PER_PACKET : usize = UDP_PACKET_SIZE_MAX/size_of::<CetiEcgSample>();
            let sample_iter = &mut samples.iter();
            while sample_count != 0 {
                let buffer : [u8; SAMPLES_PER_PACKET*size_of::<CetiEcgSample>()] = sample_iter
                    .take(sample_count.min(SAMPLES_PER_PACKET))
                    .flat_map(|sample| {
                        let bytes = sample.sys_time_us.to_le_bytes().into_iter()
                            .chain(sample.rtc_time_s.to_le_bytes().into_iter())
                            .chain(sample.ecg_readings.to_le_bytes().into_iter())
                            .chain(sample.leads_off_readings_p.to_le_bytes().into_iter())
                            .chain(sample.leads_off_readings_p.to_le_bytes().into_iter())
                            .chain(sample.sample_indexes.to_le_bytes().into_iter());
                        return bytes;
                    })
                    .collect::<Vec<u8>>()
                    .try_into()
                    .unwrap();
                
                //send to all subscribed upd addresses
                for dest_addr in dest_list.iter(){
                    socket.send_to(&buffer, dest_addr)?;
                }
                sample_count = sample_count- sample_count.min(SAMPLES_PER_PACKET);
            }
        } else {
            // there is noone subscribed to the udp stream
            if !paused {
                paused = true;
            }
            println!("Waiting on subscribers");
            sleep(Duration::from_secs(1));
        }
        //update stop flag
        stop = *stop_flag.lock().unwrap();
    }

    println!("ECG Streaming thread has been stopped");
    return Ok(())
}