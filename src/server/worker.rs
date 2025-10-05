use crossbeam::channel::Receiver;
use mio::{Events, Interest, Poll, Token};
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpStream,
    sync::Arc,
    time::Duration,
};

use crate::http::{Router, request::parse_http_request};

struct ConnState {
    stream: mio::net::TcpStream,
    write_buf: Vec<u8>,
    write_pos: usize,
}

pub fn worker_loop(id: usize, mut poll: Poll, rx: Receiver<TcpStream>, router: Arc<Router>) {
    let mut events = Events::with_capacity(1024);
    let mut token_counter = 0;
    let mut connections: HashMap<usize, ConnState> = HashMap::new();

    println!("Worker {id} started");

    loop {
        // accept new sockets
        while let Ok(stream) = rx.try_recv() {
            let token = Token(token_counter);
            let mut mio_stream = mio::net::TcpStream::from_std(stream);
            mio_stream.set_nodelay(true).ok();
            poll.registry()
                .register(&mut mio_stream, token, Interest::READABLE)
                .unwrap();

            connections.insert(
                token_counter,
                ConnState {
                    stream: mio_stream,
                    write_buf: Vec::new(),
                    write_pos: 0,
                },
            );
            token_counter += 1;
        }

        // wait for events
        if poll
            .poll(&mut events, Some(Duration::from_millis(100)))
            .is_err()
        {
            continue;
        }

        for event in &events {
            let token_id = event.token().0;

            // we’ll record what to do after releasing &mut conn
            enum Action {
                None,
                Close,
                SwitchToWrite(Vec<u8>),
                SwitchToRead,
            }

            let mut action = Action::None;

            if let Some(conn) = connections.get_mut(&token_id) {
                if event.is_readable() {
                    let mut buf = [0u8; 4096];
                    match conn.stream.read(&mut buf) {
                        Ok(0) => {
                            println!("Worker {id}: client closed (token {:?})", event.token());
                            action = Action::Close;
                        }
                        Ok(n) => {
                            if let Some(req) = parse_http_request(&buf[..n]) {
                                let resp = router.route(&req);
                                action = Action::SwitchToWrite(resp.to_bytes());
                            }
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                        Err(e) => {
                            eprintln!("Worker {id}: read error: {e}");
                            action = Action::Close;
                        }
                    }
                } else if event.is_writable() && !conn.write_buf.is_empty() {
                    match conn.stream.write(&conn.write_buf[conn.write_pos..]) {
                        Ok(0) => {}
                        Ok(n) => {
                            conn.write_pos += n;
                            if conn.write_pos >= conn.write_buf.len() {
                                conn.write_buf.clear();
                                conn.write_pos = 0;
                                action = Action::SwitchToRead;
                            }
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                        Err(e) => {
                            eprintln!("Worker {id}: write error: {e}");
                            action = Action::Close;
                        }
                    }
                }
            }

            // After borrow ends, safely perform operations that mutate the map or registry
            match action {
                Action::Close => {
                    if let Some(mut conn) = connections.remove(&token_id) {
                        let _ = poll.registry().deregister(&mut conn.stream);
                    }
                }
                Action::SwitchToWrite(buf) => {
                    if let Some(conn) = connections.get_mut(&token_id) {
                        conn.write_buf = buf;
                        conn.write_pos = 0;
                        let _ = poll.registry().reregister(
                            &mut conn.stream,
                            event.token(),
                            Interest::WRITABLE,
                        );
                    }
                }
                Action::SwitchToRead => {
                    if let Some(conn) = connections.get_mut(&token_id) {
                        let _ = poll.registry().reregister(
                            &mut conn.stream,
                            event.token(),
                            Interest::READABLE,
                        );
                    }
                }
                Action::None => {}
            }
        }
    }
}
