use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

pub struct WebServer {
    address: String,
}

impl WebServer {
    pub fn new(address: &str) -> Self {
        WebServer {
            address: address.to_string(),
        }
    }

    pub fn run(&self) {
        let listener = TcpListener::bind(&self.address).unwrap();
        println!("Server running at http://{}/", self.address);

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            self.handle_connection(stream);
        }
    }

    fn handle_connection(&self, mut stream: TcpStream) {
        let mut buffer = [0; 1024];
        stream.read(&mut buffer).unwrap();

        let request = String::from_utf8_lossy(&buffer[..]);
        let request_line = request.lines().next().unwrap_or("");

        if request_line.starts_with("GET / ") {
            self.serve_form(&mut stream);
        } else if request_line.starts_with("POST /submit ") {
            self.handle_form_submission(&mut stream, &request);
        } else {
            self.serve_404(&mut stream);
        }
    }

    fn serve_form(&self, stream: &mut TcpStream) {
        let html = fs::read_to_string("html/startup.html")
            .unwrap_or_else(|_| String::from("<h1>Error loading page</h1>"));

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\nContent-Length: {}\r\n\r\n{}",
            html.len(),
            html
        );

        stream.write_all(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }

    fn handle_form_submission(&self, stream: &mut TcpStream, request: &str) {
        // Extract the name from POST data
        let body = request.split("\r\n\r\n").nth(1).unwrap_or("");
        let name = body
            .split('&')
            .find(|param| param.starts_with("name="))
            .and_then(|param| param.strip_prefix("name="))
            .unwrap_or("Unknown");

        // URL decode the name (replace + with space and handle %XX encoding)
        let name = name.replace('+', " ");
        let name = urlencoding::decode(&name).unwrap_or(std::borrow::Cow::Borrowed(&name));

        let html = fs::read_to_string("html/response.html")
            .unwrap_or_else(|_| String::from("<h1>Error loading page</h1>"))
            .replace("{{NAME}}", &name);

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\nContent-Length: {}\r\n\r\n{}",
            html.len(),
            html
        );

        stream.write_all(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }

    fn serve_404(&self, stream: &mut TcpStream) {
        let html = fs::read_to_string("html/not_found.html")
            .unwrap_or_else(|_| String::from("<h1>404 Not Found</h1>"));

        let response = format!(
            "HTTP/1.1 404 NOT FOUND\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
            html.len(),
            html
        );

        stream.write_all(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }
}

// Simple URL decoding module
mod urlencoding {
    use std::borrow::Cow;

    pub fn decode(string: &str) -> Result<Cow<'_, str>, ()> {
        let mut result = String::new();
        let mut chars = string.chars();

        while let Some(ch) = chars.next() {
            if ch == '%' {
                let hex: String = chars.by_ref().take(2).collect();
                if hex.len() == 2 {
                    if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                        result.push(byte as char);
                    } else {
                        return Err(());
                    }
                } else {
                    return Err(());
                }
            } else {
                result.push(ch);
            }
        }

        Ok(Cow::Owned(result))
    }
}
