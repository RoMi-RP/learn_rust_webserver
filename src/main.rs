mod web_server;
mod rest_api;
mod api_client;

use web_server::WebServer;
use rest_api::RestApi;
use api_client::ApiClient;
use std::thread;
use std::time::Duration;

fn main() {
    println!("=== Rust Web Server and REST API Demo ===\n");
    
    // Uncomment the example you want to run:
    
    // Example 1: Run the HTML web server
    // run_web_server();
    
    // Example 2: Run the REST API server
    // run_rest_api();
    
    // Example 3: Run REST API server and test with client
    run_api_with_client_demo();
}

fn run_web_server() {
    println!("Starting HTML Web Server...");
    let server = WebServer::new("127.0.0.1:7878");
    println!("Open http://127.0.0.1:7878/ in your browser\n");
    server.run();
}

fn run_rest_api() {
    println!("Starting REST API Server...");
    let mut api = RestApi::new("127.0.0.1:8080");
    api.run();
}

fn run_api_with_client_demo() {
    // Start REST API server in a separate thread
    println!("Starting REST API Server on port 8080...");
    thread::spawn(|| {
        let mut api = RestApi::new("127.0.0.1:8080");
        api.run();
    });
    
    // Wait for server to start
    println!("Waiting for server to start...\n");
    thread::sleep(Duration::from_secs(2));
    
    // Create API client and test endpoints
    println!("=== Testing REST API with Client ===\n");
    let client = ApiClient::new("127.0.0.1:8080");
    
    // Test 1: Get all users
    println!("1. Getting all users:");
    match client.get_all_users() {
        Ok(users) => {
            for user in &users {
                println!("   - User #{}: {} ({})", user.id, user.name, user.email);
            }
        }
        Err(e) => println!("   Error: {}", e),
    }
    
    println!();
    
    // Test 2: Get user by ID
    println!("2. Getting user by ID (1):");
    match client.get_user_by_id(1) {
        Ok(user) => println!("   Found: {} - {}", user.name, user.email),
        Err(e) => println!("   Error: {}", e),
    }
    
    println!();
    
    // Test 3: Create new user
    println!("3. Creating new user:");
    match client.create_user("Charlie", "charlie@example.com") {
        Ok(user) => println!("   Created: User #{} - {} ({})", user.id, user.name, user.email),
        Err(e) => println!("   Error: {}", e),
    }
    
    println!();
    
    // Test 4: Get all users again to see the new user
    println!("4. Getting all users after creation:");
    match client.get_all_users() {
        Ok(users) => {
            for user in &users {
                println!("   - User #{}: {} ({})", user.id, user.name, user.email);
            }
        }
        Err(e) => println!("   Error: {}", e),
    }
    
    println!("\n=== Demo Complete ===");
    println!("REST API is still running on http://127.0.0.1:8080/");
    println!("You can test it with curl or Postman");
    println!("\nPress Ctrl+C to stop the server");
    
    // Keep main thread alive
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
