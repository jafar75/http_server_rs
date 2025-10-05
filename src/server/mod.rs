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

const THREAD_POOL_SIZE: usize = 4;

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

    // fn worker_loop(id: usize, mut poll: Poll, rx: Receiver<TcpStream>, router: Arc<Router>) {
    //     let mut events = Events::with_capacity(1024);
    //     let mut token_counter: usize = 0;
    //     let mut connections = std::collections::HashMap::new();

    //     println!("Worker {id} started");

    //     loop {
    //         // register new sockets if any
    //         while let Ok(stream) = rx.try_recv() {
    //             let token = Token(token_counter);
    //             let mut mio_stream = mio::net::TcpStream::from_std(stream);
    //             poll.registry()
    //                 .register(
    //                     &mut mio_stream,
    //                     token,
    //                     Interest::READABLE | Interest::WRITABLE,
    //                 )
    //                 .unwrap();
    //             connections.insert(token_counter, mio_stream);
    //             token_counter += 1;
    //         }

    //         // poll for events
    //         if poll
    //             .poll(&mut events, Some(Duration::from_millis(100)))
    //             .is_ok()
    //         {
    //             for event in &events {
    //                 let token = event.token();
    //                 if let Some(_conn) = connections.get_mut(&token.0) {
    //                     if event.is_readable() {
    //                         if let Some(conn) = connections.get_mut(&token.0) {
    //                             let mut buf = [0u8; 1024];
    //                             match conn.read(&mut buf) {
    //                                 Ok(0) => {
    //                                     // Connection closed by client
    //                                     // println!(
    //                                     //     "Worker {id}: connection closed (token {:?})",
    //                                     //     token
    //                                     // );
    //                                     poll.registry().deregister(conn).unwrap();
    //                                     connections.remove(&token.0);
    //                                 }
    //                                 Ok(n) => {
    //                                     if let Some(req) = parse_http_request(&buf[..n]) {
    //                                         let resp = router.route(&req);
    //                                         let raw = resp.to_bytes();
    //                                         let _ = conn.write_all(&raw);
    //                                     }
    //                                 }
    //                                 Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
    //                                 Err(e) => {
    //                                     eprintln!("Worker {id}: read error: {}", e);
    //                                     poll.registry().deregister(conn).unwrap();
    //                                     connections.remove(&token.0);
    //                                 }
    //                             }
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }
}
