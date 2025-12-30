mod web_server;
use web_server::WebServer;

fn main() {
    let server_address = "127.0.0.1:7878";
    println!("🌤️  Starting Weather Station REST API Server...");
    println!("🔗 Server will be available at: http://{}", server_address);

    let server = WebServer::new(server_address);
    server.run();
}
