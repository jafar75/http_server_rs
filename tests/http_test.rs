use std::{
    io::{Read, Write},
    net::TcpStream,
    sync::Arc,
    thread,
    time::Duration,
};

use http_server_rs::{
    http::{HttpRequest, HttpResponse, Router, request::HttpMethod, response::HttpStatusCode},
    server::Server,
};

#[test]
fn test_http_server() {
    // --- Set up router ---
    let mut router = Router::new();

    router.register("/", HttpMethod::GET, |_: &HttpRequest| {
        let mut res = HttpResponse::new(HttpStatusCode::Ok);
        res.set_header("Content-Type", "text/plain");
        res.set_content("Hello, world\n");
        res
    });

    router.register("/hello.html", HttpMethod::GET, |_: &HttpRequest| {
        let mut res = HttpResponse::new(HttpStatusCode::Ok);
        res.set_header("Content-Type", "text/html");
        res.set_content("<html><body><h1>Hello, world in HTML</h1></body></html>");
        res
    });

    let router = Arc::new(router);

    // --- Start server ---
    let server = Server::new("127.0.0.1", 4000, router.clone());
    thread::spawn(move || {
        server.run().unwrap();
    });

    // Wait a bit for server to start
    thread::sleep(Duration::from_millis(500));

    // --- Connect clients and send HTTP requests ---
    let test_cases = vec![
        ("/", "Hello, world\n"),
        (
            "/hello.html",
            "<html><body><h1>Hello, world in HTML</h1></body></html>",
        ),
    ];

    for (path, expected_body) in test_cases {
        let mut client = TcpStream::connect("127.0.0.1:4000").expect("Failed to connect");
        client.set_nonblocking(false).unwrap();

        // Construct simple HTTP GET request
        let request = format!("GET {} HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n", path);
        client.write_all(request.as_bytes()).unwrap();

        // Read response
        let mut buf = vec![0u8; 4096];
        let n = client.read(&mut buf).unwrap();
        let response_str = std::str::from_utf8(&buf[..n]).unwrap();

        // Parse HTTP response
        let response = HttpResponse::from_bytes(response_str.as_bytes()).unwrap();

        assert_eq!(response.body, expected_body);
        println!("Request to {}: received expected response", path);
    }
}
