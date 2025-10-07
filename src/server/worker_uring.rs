// server/worker_uring.rs
use crossbeam::channel::Receiver;
use io_uring::{opcode, types, IoUring};
use std::{
    collections::HashMap,
    io,
    net::TcpStream,
    os::fd::AsRawFd,
    sync::Arc,
    time::Duration,
};

use crate::{http::{request::parse_http_request, Router}, log};

const BUF_SIZE: usize = 8 * 1024;
const RING_ENTRIES: u32 = 2 * 1024;

struct ConnState {
    stream: TcpStream,
    read_buf: Vec<u8>,
    write_buf: Vec<u8>,
    write_pos: usize,
    read_outstanding: bool,
    write_outstanding: bool,
}

impl ConnState {
    fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            read_buf: vec![0u8; BUF_SIZE],
            write_buf: Vec::new(),
            write_pos: 0,
            read_outstanding: false,
            write_outstanding: false,
        }
    }
}

pub fn worker_loop(id: usize, rx: Receiver<TcpStream>, router: Arc<Router>) -> io::Result<()> {
    println!("Worker {id} (io_uring) started");

    let mut ring = IoUring::new(RING_ENTRIES)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("io_uring init error: {:?}", e)))?;

    let mut connections: HashMap<u64, ConnState> = HashMap::new();
    let mut token_counter: u64 = 1;

    loop {
        // 1) Accept new sockets
        while let Ok(stream) = rx.try_recv() {
            let token = token_counter;
            token_counter = token_counter.wrapping_add(1);

            let _ = stream.set_nonblocking(true);
            let mut conn = ConnState::new(stream);
            let fd = conn.stream.as_raw_fd();

            // Push initial READ SQE
            unsafe {
                let recv_e = opcode::Recv::new(
                    types::Fd(fd),
                    conn.read_buf.as_mut_ptr(),
                    conn.read_buf.len() as _,
                )
                .build()
                .user_data(token);

                ring.submission().push(&recv_e)
                    .map_err(|_| io::Error::new(io::ErrorKind::Other, "submission queue full"))?;
            }
            conn.read_outstanding = true;
            connections.insert(token, conn);
        }

        // 2) Submit all pending SQEs at once
        ring.submit().map_err(|e| io::Error::new(io::ErrorKind::Other, format!("submit error: {:?}", e)))?;

        // 3) Process completions
        loop {
            let cqe_opt = ring.completion().next();
            let cqe = match cqe_opt {
                Some(c) => c,
                None => break,
            };

            let user_data = cqe.user_data();
            let res = cqe.result();

            let conn = match connections.get_mut(&user_data) {
                Some(c) => c,
                None => continue, // stale completion
            };

            if res < 0 {
                let errno = -res;
                log!("Worker {id}: io_uring op error on token {}: errno={}", user_data, errno);
                connections.remove(&user_data);
                continue;
            }

            if conn.read_outstanding {
                conn.read_outstanding = false;
                let n = res as usize;

                if n == 0 {
                    log!("Worker {id}: client closed (token {})", user_data);
                    connections.remove(&user_data);
                    continue;
                }

                if let Some(req) = parse_http_request(&conn.read_buf[..n]) {
                    let resp = router.route(&req);
                    conn.write_buf = resp.to_bytes();
                    conn.write_pos = 0;

                    // Push WRITE SQE
                    let fd = conn.stream.as_raw_fd();
                    let write_len = conn.write_buf.len() - conn.write_pos;
                    if write_len > 0 {
                        unsafe {
                            let send_e = opcode::Send::new(
                                types::Fd(fd),
                                conn.write_buf.as_ptr().add(conn.write_pos) as *const _,
                                write_len as _,
                            )
                            .build()
                            .user_data(user_data);
                            ring.submission().push(&send_e)
                                .map_err(|_| io::Error::new(io::ErrorKind::Other, "submission queue full on send"))?;
                        }
                        conn.write_outstanding = true;
                    } else {
                        // nothing to send → submit another READ
                        let fd = conn.stream.as_raw_fd();
                        conn.read_outstanding = true;
                        unsafe {
                            let recv_e = opcode::Recv::new(
                                types::Fd(fd),
                                conn.read_buf.as_mut_ptr(),
                                conn.read_buf.len() as _,
                            )
                            .build()
                            .user_data(user_data);
                            ring.submission().push(&recv_e)
                                .map_err(|_| io::Error::new(io::ErrorKind::Other, "submission queue full on recv"))?;
                        }
                    }
                } else {
                    log!("Worker {id}: failed to parse request (token {}) — closing", user_data);
                    connections.remove(&user_data);
                    continue;
                }
            } else if conn.write_outstanding {
                let n = res as usize;
                conn.write_pos += n;

                let fd = conn.stream.as_raw_fd();

                if conn.write_pos >= conn.write_buf.len() {
                    // done writing
                    conn.write_buf.clear();
                    conn.write_pos = 0;
                    conn.write_outstanding = false;

                    // submit new READ
                    conn.read_outstanding = true;
                    unsafe {
                        let recv_e = opcode::Recv::new(
                            types::Fd(fd),
                            conn.read_buf.as_mut_ptr(),
                            conn.read_buf.len() as _,
                        )
                        .build()
                        .user_data(user_data);
                        ring.submission().push(&recv_e)
                            .map_err(|_| io::Error::new(io::ErrorKind::Other, "submission queue full on recv after write"))?;
                    }
                } else {
                    // partial write → submit remaining
                    let write_len = conn.write_buf.len() - conn.write_pos;
                    unsafe {
                        let send_e = opcode::Send::new(
                            types::Fd(fd),
                            conn.write_buf.as_ptr().add(conn.write_pos) as *const _,
                            write_len as _,
                        )
                        .build()
                        .user_data(user_data);
                        ring.submission().push(&send_e)
                            .map_err(|_| io::Error::new(io::ErrorKind::Other, "submission queue full on send continuation"))?;
                    }
                    conn.write_outstanding = true;
                }
            }
        } // end completions loop

        // 4) Submit any SQEs queued by completions handling
        if ring.submission().len() > 0 {
            ring.submit().map_err(|e| io::Error::new(io::ErrorKind::Other, format!("submit error: {:?}", e)))?;
        }

        // 5) Sleep briefly if idle
        if ring.submission().len() == 0 && connections.is_empty() {
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}
