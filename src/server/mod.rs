use crossbeam::channel::unbounded;
use mio::Poll;
use nix::sys::socket::{
    setsockopt,
    sockopt::{ReuseAddr, ReusePort},
};
use std::net::TcpStream;
use std::{
    net::{SocketAddr, TcpListener},
    sync::Arc,
    thread,
};

use crate::{http::Router, server::worker::worker_loop};

mod listener;
mod worker;

const THREAD_POOL_SIZE: usize = 8;

pub struct Server {
    host: String,
    port: u16,
    router: Arc<Router>,
}

impl Server {
    pub fn new(host: impl Into<String>, port: u16, router: Arc<Router>) -> Self {
        Self {
            host: host.into(),
            port,
            router,
        }
    }

    pub fn run(&self) -> std::io::Result<()> {
        // --- socket setup ---
        let addr: SocketAddr = format!("{}:{}", self.host, self.port).parse().unwrap();
        let listener = TcpListener::bind(addr)?;
        setsockopt(&listener, ReuseAddr, &true).expect("Failed to set SO_REUSEADDR");
        setsockopt(&listener, ReusePort, &true).expect("Failed to set SO_REUSEPORT");
        listener.set_nonblocking(true)?;

        // --- create channels for each worker ---
        let mut senders = Vec::with_capacity(THREAD_POOL_SIZE);
        for i in 0..THREAD_POOL_SIZE {
            let (tx, rx) = unbounded::<TcpStream>();
            senders.push(tx);
            let poll = Poll::new()?;
            let router = self.router.clone();
            thread::spawn(move || {
                worker_loop(i, poll, rx, router);
            });
        }

        // --- listener loop ---
        let listener_fd = listener.try_clone()?;
        let senders = Arc::new(senders);

        let listener_thread = thread::spawn(move || {
            listener::accept_loop(listener_fd, senders);
        });

        listener_thread.join().unwrap();
        Ok(())
    }

}
