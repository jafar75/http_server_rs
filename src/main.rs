use http_server_rs::server::Server;

fn main() -> std::io::Result<()> {
    let server = Server::new("0.0.0.0".to_string(), 8080);
    server.run()
}
