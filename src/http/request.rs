#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HttpMethod {
    GET,
    HEAD,
    POST,
    UNKNOWN,
}

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
}

impl HttpRequest {
    pub fn new(method: HttpMethod, path: String) -> Self {
        Self { method, path }
    }
}

pub fn parse_http_request(buf: &[u8]) -> Option<HttpRequest> {
    let s = std::str::from_utf8(buf).ok()?;
    let mut parts = s.split_whitespace();
    let method = match parts.next()? {
        "GET" => HttpMethod::GET,
        "HEAD" => HttpMethod::HEAD,
        "POST" => HttpMethod::POST,
        _ => HttpMethod::UNKNOWN,
    };
    let path = parts.next()?.to_string();
    Some(HttpRequest::new(method, path))
}
