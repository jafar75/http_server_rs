mod http;
mod server;

use std::{env, sync::Arc};

use http::{HttpRequest, HttpResponse, Router, request::HttpMethod, response::HttpStatusCode};
use server::Server;

mod logger;

use logger::init_logging;

use server::WorkerBackend;

#[macro_use]
mod macros;

fn main() -> std::io::Result<()> {
    init_logging();
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

    // Read backend choice from environment variable
    let backend = match env::var("WORKER_BACKEND").unwrap_or_else(|_| "epoll".to_string()).to_lowercase().as_str() {
        "epoll" => WorkerBackend::Epoll,
        "io_uring" => WorkerBackend::IoUring,
        other => {
            eprintln!("Unknown WORKER_BACKEND '{}', defaulting to IoUring", other);
            WorkerBackend::IoUring
        }
    };

    let server = Server::new("0.0.0.0".to_string(), 8080, router, backend);
    server.run()
}
