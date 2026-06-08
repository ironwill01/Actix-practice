use actix_web::{
    HttpResponse, Responder,
    http::{StatusCode, header::ContentType},
    web,
    get
};

pub mod scopes {
    
use super::*;

    pub fn default_configs(cfg: &mut web::ServiceConfig) {
        cfg
        .service(homepage);
    }

    #[get("/")]
    async fn homepage() -> impl Responder {
        HttpResponse::build(StatusCode::OK)
            .insert_header(ContentType::html())
            .body("hello to IW app")
    }
}
