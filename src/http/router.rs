use std::collections::HashMap;
use std::sync::Arc;

use crate::http::{HttpRequest, HttpResponse, request::HttpMethod, response::HttpStatusCode};

pub type Handler = Arc<dyn Fn(&HttpRequest) -> HttpResponse + Send + Sync>;

pub struct Router {
    routes: HashMap<(String, HttpMethod), Handler>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    pub fn register<F>(&mut self, path: &str, method: HttpMethod, handler: F)
    where
        F: Fn(&HttpRequest) -> HttpResponse + Send + Sync + 'static,
    {
        self.routes
            .insert((path.to_string(), method), Arc::new(handler));
    }

    pub fn route(&self, req: &HttpRequest) -> HttpResponse {
        if let Some(handler) = self.routes.get(&(req.path.clone(), req.method.clone())) {
            handler(req)
        } else {
            let mut res = HttpResponse::new(HttpStatusCode::NotFound);
            res.set_header("Content-Type", "text/plain");
            res.set_content("404 Not Found\n");
            res
        }
    }
}
