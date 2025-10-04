use http_server_rs::server::Server;

fn main() -> std::io::Result<()> {
    let server = Server::new("0.0.0.0:8080".to_string());
    server.run()
}
