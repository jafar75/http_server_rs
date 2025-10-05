use std::sync::Arc;

use http_server_rs::{
    http::{HttpRequest, HttpResponse, Router, request::HttpMethod, response::HttpStatusCode},
    server::Server,
};

fn main() -> std::io::Result<()> {
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
    let server = Server::new("0.0.0.0".to_string(), 8080, router);
    server.run()
}
