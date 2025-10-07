use crossbeam::channel::unbounded;
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

use crate::http::Router;

mod listener;
mod worker_epoll;
mod worker_uring;

const THREAD_POOL_SIZE: usize = 8;

#[derive(Clone, Copy, Debug)]
pub enum WorkerBackend {
    Epoll,
    IoUring,
}

pub struct Server {
    host: String,
    port: u16,
    router: Arc<Router>,
    backend: WorkerBackend,
}

impl Server {
    pub fn new(
        host: impl Into<String>,
        port: u16,
        router: Arc<Router>,
        backend: WorkerBackend,
    ) -> Self {
        Self {
            host: host.into(),
            port,
            router,
            backend,
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
            let router = self.router.clone();
            let backend = self.backend;
            thread::spawn(move || match backend {
                WorkerBackend::Epoll => {
                    let _ = worker_epoll::worker_loop(i, rx, router);
                }
                WorkerBackend::IoUring => {
                    let _ = worker_uring::worker_loop(i, rx, router);
                }
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
