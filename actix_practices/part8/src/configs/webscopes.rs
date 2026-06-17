use {actix_web::{
    HttpRequest, HttpResponse, Responder, Result, get, guard,
    http::header::{ContentType, ContentEncoding , self},
    http::{StatusCode},
    web::{self, ServiceConfig
    },   
    },
    serde::Serialize
};

pub mod scopes {

use super::*;

    pub fn default_config(cfg: &mut ServiceConfig) {
        cfg.service(home_page).service(
            web::resource("/")
                .name("homepage")
                .guard(guard::Get())
                .to(HttpResponse::Ok),
        )
        ;
    }

    // Just for the extra i changed the 404 page from default you can create your own response
    // with a custom function as you can see here
    pub async fn not_found() -> Result<HttpResponse> {
        Ok(HttpResponse::build(StatusCode::OK)
            .content_type(ContentType::html())
            .body("<h1>404</h1>"))
    }

    #[get("/")]
    async fn home_page() -> impl Responder {
        HttpResponse::build(StatusCode::OK)
            .content_type(ContentType::html())
            .body("Welcome to my app from IW")
    }

    #[get("")]
    async fn home_redirect(req: HttpRequest) -> Result<HttpResponse> {
        let url = req.url_for("homepage", ["/"])?;
        println!("Requset for mainpage redirecting ...");
        Ok(
            HttpResponse::Found()
            .insert_header((header::LOCATION , url.as_str()))
            .finish()
        )
    }

    pub fn custom_configs(cfg: &mut ServiceConfig) {
        cfg.service(json_resp);
    }

    // So this one is is responses how you can actually talk back to user when you send them data
    // we have few ways 

    // First one is string 
    // Require serde Serialize
    #[derive(Serialize)]
    struct JsonData {
        data : String
    }

    #[get("/json/{user}")]
    async fn json_resp(name : web::Path<String>) -> Result<impl Responder> {
        let data = JsonData { data : name.to_string() };
        Ok(web::Json(data))
    }

    // Also data can be compressed cause actix provides methods to do that using 
    // middlewares using this codecs
    // Brotli 
    // Gzip
    // Deflate 
    // Identity

    // A response's Content-Encoding header defaults to ContentEncoding::Auto, which performs automatic content compression
    // negotiation based on the request's Accept-Encoding header.
    // so for this get back to main.rs i just dropped the example there 

    // Important part is where you can actually disable this using Content-Encoding to an Identity
    // value

    #[get("/")]
    async fn data_compress() -> HttpResponse {
        HttpResponse::Ok()
        // Disable compression
        .insert_header(ContentEncoding::Identity)
        .body("Some random data for no reason !")
    }

}
