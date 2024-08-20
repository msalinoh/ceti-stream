use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

pub fn tcp_handler(
    stop_flag : Arc<Mutex<bool>>,
    audio_subs : Arc<Mutex<Vec<SocketAddr>>>,
    battery_subs : Arc<Mutex<Vec<SocketAddr>>>,
    ecg_subs : Arc<Mutex<Vec<SocketAddr>>>,
    imu_quat_subs : Arc<Mutex<Vec<SocketAddr>>>,
    imu_accel_subs : Arc<Mutex<Vec<SocketAddr>>>,
    imu_gyro_subs : Arc<Mutex<Vec<SocketAddr>>>,
    imu_mag_subs : Arc<Mutex<Vec<SocketAddr>>>,
    light_subs : Arc<Mutex<Vec<SocketAddr>>>,
    pressure_subs : Arc<Mutex<Vec<SocketAddr>>>,
) -> std::io::Result<()>{
    let listener = TcpListener::bind("0.0.0.0:8080")?;
    println!("Server listening on {:?}", listener.local_addr().unwrap());
    
    let mut stop  = *stop_flag.lock().unwrap();
    while !stop {
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    println!("New connection: {}", stream.peer_addr().unwrap());
                    let mut data = [0u8; 1500];
                    stream.read(&mut data)?;
                    let input_string = String::from_utf8_lossy(&data);
                    let mut input = input_string.split_whitespace();
                    match input.next() {
                        Some("stop") => {
                            println!("Stopping server");
                            *stop_flag.lock().unwrap() = true;
                        },
                        Some("audio") => {
                            match input.next() {
                                Some("subscribe") => {
                                    let port_number : u16 = input.next().unwrap().parse().unwrap();
                                    let socket_addr = SocketAddr::from((stream.peer_addr().unwrap().ip(), port_number));
                                    println!("{:?} was subscribed to audio stream", socket_addr);
                                    audio_subs.lock().unwrap().push(socket_addr);
                                },
                                Some("unsubscribe") => {
                                    let port_number : u16 = input.next().unwrap().parse().unwrap();
                                    let socket_addr = SocketAddr::from((stream.peer_addr().unwrap().ip(), port_number));
                                    println!("{:?} was unsubscribed to audio stream", socket_addr);
                                    audio_subs.lock().unwrap().retain(|&addr| addr != socket_addr);
                                }
                                _ => {
                                    stream.write(b"audio command options:\n");
                                    stream.write(b"  subscribe <udp_port>\n");
                                    stream.write(b"  unsubscribe <udp_port>\n");
                                }

                            }
                        },
                        _ => {
                            stream.write(b"command options:\n");
                            stream.write(b"  audio - modify audio subscription\n");
                            stream.write(b"  stop  - stop server\n");
                        },
                    }
                }
                Err(e) => {
                    println!("Error: {}", e);
                    /* connection failed */
                }
            }
        }
        stop  = *stop_flag.lock().unwrap();
    }
    println!("Subscription Handler stopped!");
    return Ok(());
}