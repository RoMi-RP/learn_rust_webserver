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
    let html = r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Name Request Form</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            max-width: 600px;
            margin: 50px auto;
            padding: 20px;
            background-color: #f0f0f0;
        }
        .container {
            background-color: white;
            padding: 30px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        h1 {
            color: #333;
        }
        input[type="text"] {
            width: 100%;
            padding: 10px;
            margin: 10px 0;
            border: 1px solid #ddd;
            border-radius: 4px;
            box-sizing: border-box;
        }
        button {
            background-color: #4CAF50;
            color: white;
            padding: 12px 20px;
            border: none;
            border-radius: 4px;
            cursor: pointer;
            font-size: 16px;
        }
        button:hover {
            background-color: #45a049;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>Welcome!</h1>
        <p>Please enter your name:</p>
        <form action="/submit" method="POST">
            <input type="text" name="name" placeholder="Enter your name" required>
            <button type="submit">Submit</button>
        </form>
    </div>
</body>
</html>"#;

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

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Greeting</title>
    <style>
        body {{
            font-family: Arial, sans-serif;
            max-width: 600px;
            margin: 50px auto;
            padding: 20px;
            background-color: #f0f0f0;
        }}
        .container {{
            background-color: white;
            padding: 30px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        h1 {{
            color: #4CAF50;
        }}
        a {{
            display: inline-block;
            margin-top: 20px;
            color: #4CAF50;
            text-decoration: none;
        }}
        a:hover {{
            text-decoration: underline;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Hello, {}!</h1>
        <p>Thank you for submitting your name.</p>
        <a href="/">← Go Back</a>
    </div>
</body>
</html>"#,
        name
    );

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\nContent-Length: {}\r\n\r\n{}",
        html.len(),
        html
    );

    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
    }

    fn serve_404(&self, stream: &mut TcpStream) {
    let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>404 Not Found</title>
</head>
<body>
    <h1>404 - Page Not Found</h1>
    <p><a href="/">Go to Home</a></p>
</body>
</html>"#;

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
