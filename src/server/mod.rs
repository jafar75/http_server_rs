use std::thread;

mod listener;

pub struct Server {
    addr: String,
}

impl Server {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }

    pub fn run(&self) -> std::io::Result<()> {
        println!("Starting server on {}", self.addr);

        // spawn a listener thread
        let addr = self.addr.clone();
        let handle = thread::spawn(move || {
            if let Err(e) = listener::run(&addr) {
                eprintln!("Listener error: {}", e);
            }
        });

        println!("Server is running. Press Ctrl+C to stop.");

        handle.join().unwrap();
        Ok(())
    }
}
