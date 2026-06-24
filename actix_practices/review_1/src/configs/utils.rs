use actix_files::NamedFile;
use actix_web::{HttpRequest, HttpResponse, Result};
use std::path::PathBuf;

pub async fn serve_template(req: HttpRequest, filename: &str) -> Result<HttpResponse> {
    let path = PathBuf::from(format!("{}/{}", super::TEMPLATE_PATH, filename));
    
    match NamedFile::open(&path) {
        Ok(file) => Ok(file.into_response(&req)),
        Err(err) => {
            eprintln!("Template not found: {} - Error: {}", path.display(), err);
            Ok(HttpResponse::NotFound().body(format!("404 - Page {} not found", filename)))
        }
    }
}