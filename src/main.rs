use std::{env, net::{SocketAddr, TcpStream}, io::{self, ErrorKind}, thread, sync::{Arc, Mutex}, time::Duration};
use std::io::Write;
use std::io::Read;

fn read_thread(conn: Arc<Mutex<TcpStream>>) -> () {

    let mut rbytes = [0u8; 100];
    loop {
        rbytes.fill(0);
        let res = conn.lock().unwrap().read(&mut rbytes);
        match res {
            Ok(len) => { 
                        if len != 0 {
                            let msg = String::from_utf8_lossy(&rbytes);
                            println!("{}", msg);
                        }
                        else {
                            println!("Disconnected!");
                            break;
                        }
                     },
            Err(err) => {
                    if ErrorKind::ConnectionReset == err.kind() {
                        println!("Server down! Reading stopped.");
                        break;
                    }
                    thread::sleep(Duration::from_secs(1));
            },
        }
    }
}

fn main() {
    println!("Hello, client!");
    let args:Vec<_> = env::args().collect();
    let port: u16;
    if args.len() == 2 {
        port = args[1].parse().unwrap();
    }
    else {
        port = 7007;
    }

    let srv_addr = SocketAddr::from(([127,0,0,1], port));
    let mut s_msg = String::from("");
    let mut uname = String::from("");
    let srv_conn = Arc::new(Mutex::new(TcpStream::connect(srv_addr).unwrap()));
    srv_conn.lock().unwrap().set_nonblocking(true).expect("Failed to set non-blocking");
    let mut r_bytes = [0; 100];
    let handle: thread::JoinHandle<()>;

    let stdin = io::stdin();
    println!("Enter user name:");
    _ = stdin.read_line(&mut uname).unwrap();
    let uname_slice = uname.trim();
    uname = uname_slice.to_string();
    uname.push_str(": ");

    let conn_copy = srv_conn.clone();
    handle = thread::spawn(move || {read_thread(conn_copy)} );
    loop {
        r_bytes.fill(0);
        s_msg.clear();
        _ = stdin.read_line(&mut s_msg).unwrap();
        let combined_str = format!("{}{}", uname, s_msg);
        print!("{}", combined_str);

        let wr_res = srv_conn.lock().unwrap().write(combined_str.as_bytes());
        if let Err(err) = wr_res {
            if ErrorKind::ConnectionReset == err.kind() {
                println!("Server down! Write stopped.");
                break;
            }
        }

        srv_conn.lock().unwrap().flush().unwrap();
        if s_msg == "quit\r\n" {
            srv_conn.lock().unwrap().shutdown(std::net::Shutdown::Both).unwrap();
            break;
        }
    }
    handle.join().unwrap();
}
