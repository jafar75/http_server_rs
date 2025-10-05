use std::io::ErrorKind;
use std::net::TcpListener;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crossbeam::channel::Sender;
use std::net::TcpStream;

pub fn accept_loop(listener: TcpListener, senders: Arc<Vec<Sender<TcpStream>>>) {
    let mut idx: usize = 0;
    loop {
        match listener.accept() {
            Ok((stream, peer)) => {
                println!("Accepted connection from {}", peer);
                stream.set_nonblocking(true).unwrap();

                // round-robin select worker
                if let Err(err) = senders[idx].send(stream) {
                    eprintln!("Failed to send stream to worker {idx}: {err}");
                }
                idx = (idx + 1) % senders.len();
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(50));
                continue;
            }
            Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => {
                eprintln!("Accept error: {}", e);
                break;
            }
        }
    }
}
