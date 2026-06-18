use {
    actix_web::{
        HttpResponse , 
        HttpRequest , 
        Result ,
        web , 
        get ,
        http::header::{
            ContentType
        }
    } ,
    askama::{
        Template
    } ,
    actix_files::{
         NamedFile ,
    } ,
    std::path::{
        PathBuf
    } ,
};

const TEMPLATE_PATH : & 'static str = "./src/templates";  

pub mod webscopes {
    use super::*;

    #[derive(Template)]
    #[template(path = "index.html")]
    struct HomePage {
        name : & 'static str
    }

    pub fn default_configs(cfg : &mut web::ServiceConfig) {
        cfg.service(
            web::scope("")
            .service(main_page)
            //.service(main_page_template)
        );
    }

    #[get("/")]
    async fn main_page(req : HttpRequest) -> Result<HttpResponse> {
        let path : PathBuf = PathBuf::from(format!("{}/index.html" , TEMPLATE_PATH));
        
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


    // this part has nothing to do with framework practice i just wanted to check template generation libs
    // in rust and then use it later on 
    //#[get("/")]
    async fn main_page_template(req : HttpRequest) -> Result<HttpResponse> {
        let page = HomePage {
            name : "Nikan"
        };

        let html = page.render().unwrap();

        Ok(
            HttpResponse::Ok()
            .content_type(ContentType::html())
            .body(html)
        )
    }
}