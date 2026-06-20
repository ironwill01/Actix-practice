//! # Defaults
//! Where all of default handles exist 

use {
    super::TEMPLATE_PATH,
    actix_web::{
        HttpResponse , HttpRequest , Result
        , get 
        , web::{
            self
        }
    },
    actix_files::{
        NamedFile
    },
    std::path::{
        PathBuf
    },
};

pub fn default_configs(cfg : &mut web::ServiceConfig) {
    cfg.service(
      web::scope("")
      .service(homepage)
      .service(favicon)
      .service(robots)
    );
}

// Loading homepage as file ( no askama here atleast for now )
#[get("/")]
async fn homepage(req : HttpRequest) -> Result<HttpResponse> {
    let homepage_file = PathBuf::from(format!("{}/homepage.html" , TEMPLATE_PATH));

    let response = match NamedFile::open(homepage_file) {
        Ok(page) => {
            page.into_response(&req)
        } , 
        Err(err) => {
            eprintln!("Error : {}" , err);
            return Ok(HttpResponse::InternalServerError().body("<h1>Error code 500</h1>"));
        }
    };

    Ok(response)
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
