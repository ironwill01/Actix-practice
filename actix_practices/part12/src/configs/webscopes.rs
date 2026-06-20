use {
    actix_web::{
        HttpResponse , 
        HttpRequest , 
        Result ,
        Error ,
        web ,
        get ,
        http::header::{
            ContentType ,
            ContentDisposition , 
            DispositionType
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

// Saved the template path 
const TEMPLATE_PATH : & 'static str = "./actix_practices/part12/src/templates";

pub mod webscopes {
    use super::*;

    // Askama lib practice dont even releated to this part ignore it
    #[derive(Template)]
    #[template(path = "index.html")]
    struct HomePage {
        name : & 'static str
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

    // The actual part where we work on statics 

    pub fn default_configs(cfg : &mut web::ServiceConfig) {
        cfg.service(
            web::scope("")
            .service(main_page)
            .service(index)
            .service(favicon)
            .service(robots)
        );
    }

    // It is possible to serve static files with a custom path pattern and NamedFile. To match a path tail, we can use a [.*] regex.
    // Although i changed code here a bit from actix to use rust error handling   

    #[get("/")]
    async fn main_page(req : HttpRequest) -> Result<HttpResponse> {
        let path : PathBuf = PathBuf::from(format!("{}/homepage.html" , TEMPLATE_PATH));

        let response = match NamedFile::open(path) {
            Ok(page) => {
                // Also NamedFile have impl Response so you can directly turn it into an HttpResponse
                page.into_response(&req)
            } ,
            Err(_err) => {
                return Ok(HttpResponse::InternalServerError().body("<h1>InternalServerError 500</h1>"))
            }
        };
        
        Ok(response)
    }


    // This is direct copy of what we have in actix doc
    #[get("/{filename:.*}")]
    async fn index(req: HttpRequest) -> actix_web::Result<NamedFile> {
        let path: PathBuf = req.match_info().query("filename").parse().unwrap();
        // Because my HTML files are somewhere else i had to add this path to the function
        let full_path = PathBuf::from(TEMPLATE_PATH).join(&path); 

        // Block path traversal attempts
        if path.components().any(|c| c == std::path::Component::ParentDir) {
            return Err(actix_web::error::ErrorForbidden("Invalid path"));
        }
        
        Ok(NamedFile::open(full_path)?)
    }

    // Warning from the guide
    // Matching a path tail with the [.*] regex and using it to return a NamedFile has serious security implications. 
    // It offers the possibility for an attacker to insert ../ 
    // into the URL and access every file on the host that the user running the server has access to.


    // Also one more thing , in modern browser we have two extra calls one is for favicon.ico
    #[get("/favicon.ico")]
    async fn favicon(req : HttpRequest) -> Result<HttpResponse> {
        Ok(NamedFile::open(format!("{}/favicon.ico" , TEMPLATE_PATH)).unwrap().into_response(&req))
    }

    // and other is for robots.txt which is basically crawlers
    #[get("/robots.txt")]
    async fn robots() -> HttpResponse {
        HttpResponse::Ok()
            .content_type("text/plain")
            .body("User-agent: *\nAllow: /")
    }

    
    // Now about configuration 

    // Refrencing from the actix doc

    // NamedFiles can specify various options for serving files :
    // set_content_disposition - function to be used for mapping file's mime to corresponding Content-Disposition type
    // use_etag - specifies whether ETag shall be calculated and included in headers.
    // use_last_modified - specifies whether file modified timestamp should be used and added to Last-Modified header.

    // All of the above methods are optional and provided with the best defaults, But it is possible to customize any of them.
    #[get("/{filename:.*}")]
    async fn index_detailed(req : HttpRequest) -> Result<actix_files::NamedFile , Error> {
        let path : std::path::PathBuf = req.match_info().query("filename").parse().unwrap();
        
        let file = actix_files::NamedFile::open(path)?;

        Ok(
            file
            .use_last_modified(true)
            .set_content_disposition(ContentDisposition {
                disposition : DispositionType::Attachment ,
                parameters : vec![],
            })
        )
    }

}