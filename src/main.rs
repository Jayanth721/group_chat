use std::{env, net::{TcpListener, SocketAddr, TcpStream}, io::{Read, Write}, thread, sync::{Arc, Mutex}, time::Duration};

fn handle_client(stream: Arc<Mutex<TcpStream>>, all_conns: Arc<Mutex<Vec<Arc<Mutex<TcpStream>>>>>) -> i16 {
    let mut rbytes = [0u8; 100];
    let mut bytes_received;
    let this_port  = stream.lock().unwrap().peer_addr().unwrap().port();
    loop {
        rbytes.fill(0);
        let read_res = stream.lock().unwrap().read(&mut rbytes);
        match read_res {
            Ok(bytes_rec) => bytes_received = bytes_rec,
            Err(_) => { thread::sleep(Duration::from_millis(500)); continue },
        }
        println!("Server Listening, read {} bytes", bytes_received);
        let msg = String::from_utf8_lossy(&rbytes);

        // 0 bytes read generally means disconnect
        if bytes_received == 0 {
            // remove from the list & quit
            let idx = all_conns.lock().unwrap().iter().position(|x| x.lock().unwrap().peer_addr().unwrap().port() == this_port).unwrap();
            all_conns.lock().unwrap().remove(idx);
            println!("Port [{}] exited from chat", this_port);
            break;
        }

        // TODO: fix this
        if msg == "quit\r\n" {
            println!("Exit on request for port [{}]", this_port);
            stream.lock().unwrap().shutdown(std::net::Shutdown::Both).expect("Failed to shutdown.");
            break;
        }

        if msg == "^]" {
            println!("Exit on request");
            break;
        }

        for one_stream in all_conns.lock().unwrap().iter() {
            let other_port = one_stream.lock().unwrap().peer_addr().unwrap().port();

            if other_port != this_port {
                println!("Sending {} for {}", msg, other_port);
                //send to other clients only
                one_stream.lock().unwrap().write(msg.as_bytes()).unwrap();
                one_stream.lock().unwrap().flush().unwrap();
            }
        }
    }
    return 0;
}

fn main() {
    println!("Hello, world!");
    let args: Vec<_>= env::args().collect();
    let all_streams: Arc<Mutex<Vec<Arc<Mutex<TcpStream>>>>> = Arc::new(Mutex::new(Vec::new()));
    let mut handles: Vec<thread::JoinHandle<i16>>= Vec::new();
    let port: u16;
    if args.len() == 2 {
        port = args[1].parse().unwrap();
    }
    else {
        port = 7007;
    }

    let conn_addr = SocketAddr::from(([127,0,0,1], port));
    let conn_handle = TcpListener::bind(conn_addr).expect("Failed to bind.");

    for stream in conn_handle.incoming() {
        let stream_val = Arc::new(Mutex::new(stream.unwrap()));
        stream_val.lock().unwrap().set_nonblocking(true).expect("Failed to set non-blocking");
        all_streams.lock().unwrap().push(stream_val.clone());
        let all_conn = all_streams.clone();
        let this_stream = stream_val.clone();
        let handle = thread::spawn(move || { handle_client(this_stream.clone(), all_conn.clone()) });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
