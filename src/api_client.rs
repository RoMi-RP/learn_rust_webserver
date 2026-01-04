use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::TcpStream;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: String,
}

pub struct ApiClient {
    base_url: String,
}

impl ApiClient {
    pub fn new(base_url: &str) -> Self {
        ApiClient {
            base_url: base_url.to_string(),
        }
    }

    pub fn get_all_users(&self) -> Result<Vec<User>, String> {
        let response = self.make_request("GET", "/api/users", None)?;
        let api_response: ApiResponse<Vec<User>> =
            serde_json::from_str(&response).map_err(|e| e.to_string())?;

        if api_response.success {
            Ok(api_response.data.unwrap_or_default())
        } else {
            Err(api_response.message)
        }
    }

    pub fn get_user_by_id(&self, id: u32) -> Result<User, String> {
        let endpoint = format!("/api/users/{}", id);
        let response = self.make_request("GET", &endpoint, None)?;
        let api_response: ApiResponse<User> =
            serde_json::from_str(&response).map_err(|e| e.to_string())?;

        if api_response.success {
            api_response.data.ok_or_else(|| "No data returned".to_string())
        } else {
            Err(api_response.message)
        }
    }

    pub fn create_user(&self, name: &str, email: &str) -> Result<User, String> {
        let user = User {
            id: 0, // Will be assigned by server
            name: name.to_string(),
            email: email.to_string(),
        };

        let body = serde_json::to_string(&user).map_err(|error| error.to_string())?;
        let response = self.make_request("POST", "/api/users", Some(&body))?;

        println!("Raw response: {}", response);

        let api_response: ApiResponse<User> =
            serde_json::from_str(&response).map_err(|e| {
                format!("JSON parse error: {}. Response was: '{}'", e, response)
            })?;

        if api_response.success {
            api_response.data.ok_or_else(|| "No data returned".to_string())
        } else {
            Err(api_response.message)
        }
    }

    fn make_request(&self, method: &str, endpoint: &str, body: Option<&str>) -> Result<String, String> {
        // Parse base URL to get host and port
        let url = self.base_url.replace("http://", "");
        let mut stream = TcpStream::connect(&url).map_err(|e| format!("Connection failed: {}", e))?;

        // Build HTTP request
        let content_length = body.map(|b| b.len()).unwrap_or(0);
        let request = if let Some(body_content) = body {
            format!(
                "{} {} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                method, endpoint, url, content_length, body_content
            )
        } else {
            format!(
                "{} {} HTTP/1.1\r\nHost: {}\r\n\r\n",
                method, endpoint, url
            )
        };

        // Send request
        stream.write_all(request.as_bytes()).map_err(|e| e.to_string())?;
        stream.flush().map_err(|e| e.to_string())?;

        // Read response
        let mut buffer = Vec::new();
        let mut temp_buffer = [0u8; 4096];
        
        loop {
            match stream.read(&mut temp_buffer) {
                Ok(0) => break, // Connection closed
                Ok(n) => buffer.extend_from_slice(&temp_buffer[..n]),
                Err(e) => {
                    if buffer.is_empty() {
                        return Err(format!("Read error: {}", e));
                    }
                    break;
                }
            }
            
            // Check if we've received the full response
            if buffer.len() > 4 {
                let response_str = String::from_utf8_lossy(&buffer);
                if let Some(headers_end) = response_str.find("\r\n\r\n") {
                    let headers = &response_str[..headers_end];
                    if let Some(content_length) = Self::extract_content_length(headers) {
                        let body_start = headers_end + 4;
                        if buffer.len() >= body_start + content_length {
                            break;
                        }
                    }
                }
            }
        }

        let response = String::from_utf8_lossy(&buffer).to_string();

        // Extract body from response
        if let Some(body_start) = response.find("\r\n\r\n") {
            Ok(response[body_start + 4..].trim_end_matches('\0').to_string())
        } else {
            Err("Invalid HTTP response".to_string())
        }
    }

    fn extract_content_length(headers: &str) -> Option<usize> {
        for line in headers.lines() {
            if line.to_lowercase().starts_with("content-length:") {
                return line.split(':').nth(1)?.trim().parse().ok();
            }
        }
        None
    }
}
