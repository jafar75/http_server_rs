use std::io;
use std::net::TcpListener;

pub fn run(addr: &str) -> io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    println!("Listening on {}", addr);

    loop {
        match listener.accept() {
            Ok((stream, peer)) => {
                println!("Accepted connection from {}", peer);
                drop(stream); // just close immediately for now
            }
            Err(e) => eprintln!("Accept error: {}", e),
        }
    }
}
