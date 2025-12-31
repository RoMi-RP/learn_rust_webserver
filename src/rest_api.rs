use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: String,
}

pub struct RestApi {
    address: String,
    users: Vec<User>,
}

impl RestApi {
    pub fn new(address: &str) -> Self {
        RestApi {
            address: address.to_string(),
            users: vec![
                User {
                    id: 1,
                    name: "Alice".to_string(),
                    email: "alice@example.com".to_string(),
                },
                User {
                    id: 2,
                    name: "Bob".to_string(),
                    email: "bob@example.com".to_string(),
                },
            ],
        }
    }

    pub fn run(&mut self) {
        let listener = TcpListener::bind(&self.address).unwrap();
        println!("REST API running at http://{}/", self.address);
        println!("Available endpoints:");
        println!("  GET  /api/users       - Get all users");
        println!("  GET  /api/users/:id   - Get user by ID");
        println!("  POST /api/users       - Create new user");

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            self.handle_request(stream);
        }
    }

    fn handle_request(&mut self, mut stream: TcpStream) {
        let mut buffer = [0; 2048];
        stream.read(&mut buffer).unwrap();

        let request = String::from_utf8_lossy(&buffer[..]);
        let request_line = request.lines().next().unwrap_or("");

        if request_line.starts_with("GET /api/users HTTP") {
            self.get_all_users(&mut stream);
        } else if request_line.starts_with("GET /api/users/") {
            let id = self.extract_id_from_path(request_line);
            self.get_user_by_id(&mut stream, id);
        } else if request_line.starts_with("POST /api/users HTTP") {
            let body = request.split("\r\n\r\n").nth(1).unwrap_or("");
            self.create_user(&mut stream, body);
        } else {
            self.send_404(&mut stream);
        }
    }

    fn extract_id_from_path(&self, request_line: &str) -> Option<u32> {
        request_line
            .split_whitespace()
            .nth(1)?
            .split('/')
            .last()?
            .parse()
            .ok()
    }

    fn get_all_users(&self, stream: &mut TcpStream) {
        let response = ApiResponse {
            success: true,
            data: Some(&self.users),
            message: "Users retrieved successfully".to_string(),
        };

        self.send_json_response(stream, 200, &response);
    }

    fn get_user_by_id(&self, stream: &mut TcpStream, id: Option<u32>) {
        match id {
            Some(id) => {
                if let Some(user) = self.users.iter().find(|u| u.id == id) {
                    let response = ApiResponse {
                        success: true,
                        data: Some(user),
                        message: "User found".to_string(),
                    };
                    self.send_json_response(stream, 200, &response);
                } else {
                    let response: ApiResponse<User> = ApiResponse {
                        success: false,
                        data: None,
                        message: format!("User with id {} not found", id),
                    };
                    self.send_json_response(stream, 404, &response);
                }
            }
            None => {
                let response: ApiResponse<User> = ApiResponse {
                    success: false,
                    data: None,
                    message: "Invalid user ID".to_string(),
                };
                self.send_json_response(stream, 400, &response);
            }
        }
    }

    fn create_user(&mut self, stream: &mut TcpStream, body: &str) {
        match serde_json::from_str::<User>(body) {
            Ok(mut user) => {
                user.id = self.users.len() as u32 + 1;
                self.users.push(user.clone());

                let response = ApiResponse {
                    success: true,
                    data: Some(user),
                    message: "User created successfully".to_string(),
                };
                self.send_json_response(stream, 201, &response);
            }
            Err(_) => {
                let response: ApiResponse<User> = ApiResponse {
                    success: false,
                    data: None,
                    message: "Invalid JSON data".to_string(),
                };
                self.send_json_response(stream, 400, &response);
            }
        }
    }

    fn send_json_response<T: Serialize>(&self, stream: &mut TcpStream, status: u16, data: &T) {
        let json = serde_json::to_string(data).unwrap();
        let status_text = match status {
            200 => "OK",
            201 => "Created",
            400 => "Bad Request",
            404 => "Not Found",
            _ => "Unknown",
        };

        let response = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
            status, status_text, json.len(), json
        );

        stream.write_all(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }

    fn send_404(&self, stream: &mut TcpStream) {
        let response: ApiResponse<String> = ApiResponse {
            success: false,
            data: None,
            message: "Endpoint not found".to_string(),
        };
        self.send_json_response(stream, 404, &response);
    }
}
