mod web_server;
use web_server::WebServer;

fn main() {
    let server = WebServer::new("127.0.0.1:7878");
    server.run();
}
