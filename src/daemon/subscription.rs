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
                        Some("battery") => {
                            match input.next() {
                                Some("subscribe") => {
                                    let port_number : u16 = input.next().unwrap().parse().unwrap();
                                    let socket_addr = SocketAddr::from((stream.peer_addr().unwrap().ip(), port_number));
                                    println!("{:?} was subscribed to battery stream", socket_addr);
                                    audio_subs.lock().unwrap().push(socket_addr);
                                },
                                Some("unsubscribe") => {
                                    let port_number : u16 = input.next().unwrap().parse().unwrap();
                                    let socket_addr = SocketAddr::from((stream.peer_addr().unwrap().ip(), port_number));
                                    println!("{:?} was unsubscribed to battery stream", socket_addr);
                                    audio_subs.lock().unwrap().retain(|&addr| addr != socket_addr);
                                }
                                _ => {
                                    stream.write(b"battery command options:\n");
                                    stream.write(b"  subscribe <udp_port>\n");
                                    stream.write(b"  unsubscribe <udp_port>\n");
                                }
                            }
                        },
                        Some("ecg") => {
                            match input.next() {
                                Some("subscribe") => {
                                    let port_number : u16 = input.next().unwrap().parse().unwrap();
                                    let socket_addr = SocketAddr::from((stream.peer_addr().unwrap().ip(), port_number));
                                    println!("{:?} was subscribed to ecg stream", socket_addr);
                                    audio_subs.lock().unwrap().push(socket_addr);
                                },
                                Some("unsubscribe") => {
                                    let port_number : u16 = input.next().unwrap().parse().unwrap();
                                    let socket_addr = SocketAddr::from((stream.peer_addr().unwrap().ip(), port_number));
                                    println!("{:?} was unsubscribed to ecg stream", socket_addr);
                                    audio_subs.lock().unwrap().retain(|&addr| addr != socket_addr);
                                }
                                _ => {
                                    stream.write(b"ecg command options:\n");
                                    stream.write(b"  subscribe <udp_port>\n");
                                    stream.write(b"  unsubscribe <udp_port>\n");
                                }
                            }
                        },
                        Some("imu_quat")=> {
                            match input.next() {
                                Some("subscribe") => {
                                    let port_number : u16 = input.next().unwrap().parse().unwrap();
                                    let socket_addr = SocketAddr::from((stream.peer_addr().unwrap().ip(), port_number));
                                    println!("{:?} was subscribed to imu quaternion stream", socket_addr);
                                    imu_quat_subs.lock().unwrap().push(socket_addr);
                                },
                                Some("unsubscribe") => {
                                    let port_number : u16 = input.next().unwrap().parse().unwrap();
                                    let socket_addr = SocketAddr::from((stream.peer_addr().unwrap().ip(), port_number));
                                    println!("{:?} was unsubscribed to imu quaternion stream", socket_addr);
                                    imu_quat_subs.lock().unwrap().retain(|&addr| addr != socket_addr);
                                }
                                _ => {
                                    stream.write(b"imu_quat command options:\n");
                                    stream.write(b"  subscribe <udp_port>\n");
                                    stream.write(b"  unsubscribe <udp_port>\n");
                                }
                            }
                        },
                        Some("imu_accel")=> {
                            match input.next() {
                                Some("subscribe") => {
                                    let port_number : u16 = input.next().unwrap().parse().unwrap();
                                    let socket_addr = SocketAddr::from((stream.peer_addr().unwrap().ip(), port_number));
                                    println!("{:?} was subscribed to accelerometer stream", socket_addr);
                                    imu_accel_subs.lock().unwrap().push(socket_addr);
                                },
                                Some("unsubscribe") => {
                                    let port_number : u16 = input.next().unwrap().parse().unwrap();
                                    let socket_addr = SocketAddr::from((stream.peer_addr().unwrap().ip(), port_number));
                                    println!("{:?} was unsubscribed to accelerometer stream", socket_addr);
                                    imu_accel_subs.lock().unwrap().retain(|&addr| addr != socket_addr);
                                }
                                _ => {
                                    stream.write(b"imu_accel command options:\n");
                                    stream.write(b"  subscribe <udp_port>\n");
                                    stream.write(b"  unsubscribe <udp_port>\n");
                                }
                            }
                        },
                        Some("imu_gyro")=> {
                            match input.next() {
                                Some("subscribe") => {
                                    let port_number : u16 = input.next().unwrap().parse().unwrap();
                                    let socket_addr = SocketAddr::from((stream.peer_addr().unwrap().ip(), port_number));
                                    println!("{:?} was subscribed to gyroscope stream", socket_addr);
                                    imu_gyro_subs.lock().unwrap().push(socket_addr);
                                },
                                Some("unsubscribe") => {
                                    let port_number : u16 = input.next().unwrap().parse().unwrap();
                                    let socket_addr = SocketAddr::from((stream.peer_addr().unwrap().ip(), port_number));
                                    println!("{:?} was unsubscribed to gyroscope stream", socket_addr);
                                    imu_gyro_subs.lock().unwrap().retain(|&addr| addr != socket_addr);
                                }
                                _ => {
                                    stream.write(b"imu_gyro command options:\n");
                                    stream.write(b"  subscribe <udp_port>\n");
                                    stream.write(b"  unsubscribe <udp_port>\n");
                                }
                            }
                        },
                        Some("imu_mag")=> {
                            match input.next() {
                                Some("subscribe") => {
                                    let port_number : u16 = input.next().unwrap().parse().unwrap();
                                    let socket_addr = SocketAddr::from((stream.peer_addr().unwrap().ip(), port_number));
                                    println!("{:?} was subscribed to magnetometer stream", socket_addr);
                                    imu_accel_subs.lock().unwrap().push(socket_addr);
                                },
                                Some("unsubscribe") => {
                                    let port_number : u16 = input.next().unwrap().parse().unwrap();
                                    let socket_addr = SocketAddr::from((stream.peer_addr().unwrap().ip(), port_number));
                                    println!("{:?} was unsubscribed to magnetometer stream", socket_addr);
                                    imu_accel_subs.lock().unwrap().retain(|&addr| addr != socket_addr);
                                }
                                _ => {
                                    stream.write(b"imu_mag command options:\n");
                                    stream.write(b"  subscribe <udp_port>\n");
                                    stream.write(b"  unsubscribe <udp_port>\n");
                                }
                            }
                        },
                        Some("light")=> {
                            match input.next() {
                                Some("subscribe") => {
                                    let port_number : u16 = input.next().unwrap().parse().unwrap();
                                    let socket_addr = SocketAddr::from((stream.peer_addr().unwrap().ip(), port_number));
                                    println!("{:?} was subscribed to ambient light sensor stream", socket_addr);
                                    imu_accel_subs.lock().unwrap().push(socket_addr);
                                },
                                Some("unsubscribe") => {
                                    let port_number : u16 = input.next().unwrap().parse().unwrap();
                                    let socket_addr = SocketAddr::from((stream.peer_addr().unwrap().ip(), port_number));
                                    println!("{:?} was unsubscribed to ambient light sensor stream", socket_addr);
                                    imu_accel_subs.lock().unwrap().retain(|&addr| addr != socket_addr);
                                }
                                _ => {
                                    stream.write(b"light command options:\n");
                                    stream.write(b"  subscribe <udp_port>\n");
                                    stream.write(b"  unsubscribe <udp_port>\n");
                                }
                            }
                        },
                        Some("pressure")=> {
                            match input.next() {
                                Some("subscribe") => {
                                    let port_number : u16 = input.next().unwrap().parse().unwrap();
                                    let socket_addr = SocketAddr::from((stream.peer_addr().unwrap().ip(), port_number));
                                    println!("{:?} was subscribed to pressure sensor stream", socket_addr);
                                    imu_accel_subs.lock().unwrap().push(socket_addr);
                                },
                                Some("unsubscribe") => {
                                    let port_number : u16 = input.next().unwrap().parse().unwrap();
                                    let socket_addr = SocketAddr::from((stream.peer_addr().unwrap().ip(), port_number));
                                    println!("{:?} was unsubscribed to pressure sensor stream", socket_addr);
                                    imu_accel_subs.lock().unwrap().retain(|&addr| addr != socket_addr);
                                }
                                _ => {
                                    stream.write(b"pressure command options:\n");
                                    stream.write(b"  subscribe <udp_port>\n");
                                    stream.write(b"  unsubscribe <udp_port>\n");
                                }
                            }
                        },
                        _ => {
                            stream.write(b"command options:\n");
                            stream.write(b"  audio     - modify audio subscription\n");
                            stream.write(b"  batery    - modify battery subscription\n");
                            stream.write(b"  imu_quat  - modify imu quaternion subscription\n");
                            stream.write(b"  imu_accel - modify imu accelerometer subscription\n");
                            stream.write(b"  imu_gyro  - modify imu gyroscope subscription\n");
                            stream.write(b"  imu_mag   - modify imu magnetometer subscription\n");
                            stream.write(b"  light     - modify ambient light sensor subscription\n");
                            stream.write(b"  pressure  - modify pressure sensor subscription\n");
                            stream.write(b"  stop      - stop server\n");
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