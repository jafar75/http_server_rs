use std::{
    io::{Read, Write},
    net::TcpStream,
    thread,
    time::Duration,
};

use http_server_rs::server::Server;

#[test]
fn test_echo_server() {
    // --- start server in background thread ---
    let server = Server::new("127.0.0.1", 4000);
    thread::spawn(move || {
        server.run().unwrap();
    });

    // wait a bit for server to start
    thread::sleep(Duration::from_millis(500));

    // --- connect clients ---
    let mut clients = vec![];
    for _ in 0..3 {
        let stream = TcpStream::connect("127.0.0.1:4000").expect("Failed to connect");
        stream.set_nonblocking(false).unwrap();
        clients.push(stream);
    }

    // --- send and receive messages ---
    let messages = vec![
        "Hello from client 1",
        "Hello from client 2",
        "Hello from client 3",
    ];

    for (i, client) in clients.iter_mut().enumerate() {
        let msg = messages[i];
        client.write_all(msg.as_bytes()).unwrap();

        let mut buf = vec![0u8; 1024];
        let n = client.read(&mut buf).unwrap();
        let echoed = std::str::from_utf8(&buf[..n]).unwrap();

        assert_eq!(echoed, msg);
        println!("Client {}: sent and received '{}'", i + 1, echoed);
    }
}
