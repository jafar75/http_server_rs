use std::{fmt::Write, num::ParseIntError};

#[derive(Debug, Clone, Copy)]
pub enum HttpStatusCode {
    Ok = 200,
    NotFound = 404,
    BadRequest = 400,
}

impl HttpStatusCode {
    pub fn from_u16(code: u16) -> Result<Self, String> {
        match code {
            200 => Ok(HttpStatusCode::Ok),
            400 => Ok(HttpStatusCode::BadRequest),
            404 => Ok(HttpStatusCode::NotFound),
            _ => Err(format!("Unknown HTTP status code: {}", code)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: HttpStatusCode,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl HttpResponse {
    pub fn new(status: HttpStatusCode) -> Self {
        Self {
            status,
            headers: Vec::new(),
            body: String::new(),
        }
    }

    pub fn set_header(&mut self, key: &str, val: &str) {
        self.headers.push((key.to_string(), val.to_string()));
    }

    pub fn set_content(&mut self, body: impl Into<String>) {
        self.body = body.into();
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut res = String::new();
        let status_line = match self.status {
            HttpStatusCode::Ok => "200 OK",
            HttpStatusCode::NotFound => "404 Not Found",
            HttpStatusCode::BadRequest => "400 Bad Request",
        };
        write!(&mut res, "HTTP/1.1 {}\r\n", status_line).unwrap();
        for (k, v) in &self.headers {
            write!(&mut res, "{}: {}\r\n", k, v).unwrap();
        }
        write!(&mut res, "Content-Length: {}\r\n", self.body.len()).unwrap();
        res.push_str("\r\n");
        res.push_str(&self.body);
        res.into_bytes()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let s = std::str::from_utf8(bytes).map_err(|e| e.to_string())?;
        let parts: Vec<&str> = s.split("\r\n\r\n").collect();
        if parts.len() != 2 {
            return Err("Invalid HTTP response format".into());
        }

        let header_lines: Vec<&str> = parts[0].lines().collect();
        if header_lines.is_empty() {
            return Err("Empty HTTP response".into());
        }

        // Parse status line
        let status_line = header_lines[0];
        let status_parts: Vec<&str> = status_line.split_whitespace().collect();
        if status_parts.len() < 2 {
            return Err("Invalid status line".into());
        }
        let status_code: u16 = status_parts[1]
            .parse()
            .map_err(|e: ParseIntError| e.to_string())?;
        let status = HttpStatusCode::from_u16(status_code)?;

        // Parse headers
        let mut response = HttpResponse::new(status);
        for line in &header_lines[1..] {
            if let Some((key, value)) = line.split_once(":") {
                response.set_header(key.trim(), value.trim());
            }
        }

        // Set body
        response.set_content(parts[1]);

        Ok(response)
    }
}
