//! # Defaults
//! Where all of default handles exist 

use {
    super::{
        TEMPLATE_PATH ,
        UserState ,
        UserTemplate
    } , 
    actix_files::NamedFile ,
    actix_web::{
        HttpRequest, 
        HttpResponse, 
        Result, 
        get,
        http::header::ContentType,
        web::{
            self 
        }
    } , 
    askama::Template ,
};

pub fn default_configs(cfg : &mut web::ServiceConfig) {
    cfg
    .service(homepage)
    .service(favicon)
    .service(robots);
}

// Loading homepage as file ( no askama here atleast for now )
#[get("/")]
async fn homepage(_req : HttpRequest , state : web::Data<UserState>) -> Result<HttpResponse> {

    let messages = UserTemplate::new(&state);

    let html = match messages.render() {
        Ok(body) => body ,
        Err(err) => {
            eprintln!("Error : {}" , err);
            return Ok(HttpResponse::InternalServerError().body("<h1> Error 500 </h1>"));
        }   
    };

    Ok(
        HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html)
    )
}

// Setting icon 
#[get("/favicon.ico")]
async fn favicon(req : HttpRequest) -> Result<HttpResponse> {
    Ok(NamedFile::open(format!("{}/favicon.ico" , TEMPLATE_PATH)).unwrap().into_response(&req))
}

// Robot perm for website
#[get("/robots.txt")]
async fn robots() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/plain")
        .body("User-agent: *\nAllow: /")
}
