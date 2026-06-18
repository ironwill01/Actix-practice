use {
    actix_web::{
        HttpResponse , 
        HttpRequest , 
        Result ,
        web , 
        get
    } ,
    actix_files::{
         NamedFile ,
    } ,
    std::path::{
        PathBuf
    } ,
};

pub mod webscopes {
    use super::*;

    pub fn default_configs(cfg : &mut web::ServiceConfig) {
        cfg.service(
            web::scope("")
            .service(main_page)
        );
    }

    #[get("/")]
    async fn main_page(req : HttpRequest) -> Result<HttpResponse> {
        let path : PathBuf = PathBuf::from("./actix_practices/static/html/homepage.html");

        let response = match NamedFile::open(path) {
            Ok(page) => {
                page.into_response(&req)
            } ,
            Err(_err) => {
                return Ok(HttpResponse::InternalServerError().body("<h1>InternalServerError 500</h1>"))
            }
        };
        
        Ok(response)
    }
}