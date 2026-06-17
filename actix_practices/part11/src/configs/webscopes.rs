use {
    actix_cors::{
        Cors
    },
    actix_web::{
        HttpResponse , Result , Error , HttpRequest ,
        http::{
            header::{
                self,
                ContentType
            } , 
            StatusCode
        },
        web , get , 
        post , 
        guard ,
    },
    serde::{
        Serialize,
        Deserialize
    },
};


pub mod scopes {
    use super::*;

    pub fn default_config(cfg: &mut web::ServiceConfig) {
        cfg.service(homepage)
        .service(home_redirect);
    }

    // never knew you can name handles
    #[get("/homepage" , name = "homepage")]
    async fn homepage() -> Result<HttpResponse> {
        Ok(
            HttpResponse::build(StatusCode::OK)
            .insert_header(ContentType::html())
            .body("<h1>Hello to IW-APP</h1>")
        )
    }

    #[get("/")]
    async fn home_redirect(req : HttpRequest) -> Result<HttpResponse> {
        let url = req.url_for("homepage",[""])?; // for some reason i decided to generate URL for default page now
        println!("Redirecting to homepage ...");
        Ok(
            HttpResponse::Found()
            .insert_header((header::LOCATION , url.as_str()))
            .finish()
        )
    }

    // So we go with CORS one of the general concepts of backend dev
    // CORS It's a browser security mechanism that determines whether a 
    // web page from one origin is allowed to make requests to another origin.

    // https://example.com:443
    // │      │          │
    // │      │          └── Port
    // │      └───────────── Host
    // └──────────────────── Protocol

    // | URL                                                | Same origin?          |
    // | -------------------------------------------------- | --------------------  |
    // | `https://example.com` → `https://example.com`      | ✅ Yes                |
    // | `https://example.com` → `http://example.com`       | ❌ Different protocol |
    // | `https://example.com` → `https://api.example.com`  | ❌ Different host     |
    // | `https://example.com` → `https://example.com:3000` | ❌ Different port     |

    // so assume that now you want to login into https//bank.com
    // your browser stores a session cookie.

    // Now you visit
    // https://evil.com

    // Without CORS, JavaScript on evil.com could simply do :
    // fetch("https://bank.com/account");

    // and read :
    // {
    //     "balance": 900000,
    //     "password": "...",
    //     ...
    // }

    // allow one frontend origin

    // Since we have to do it in main ill do i there instead 

    fn cors_conf_single() -> Cors {
        Cors::default()
        .allowed_origin("https://www.myapp.test")
        .allowed_methods(vec!["GET", "POST"])
        .allowed_headers(vec![header::AUTHORIZATION, header::CONTENT_TYPE])
        .max_age(3600)
    }

    // Allow several frontend origin

    fn cors_conf_serval() -> Cors {
        Cors::default()
        .allowed_origin("https://www.myapp.test")
        .allowed_origin("https://admin.example.com")
        .allowed_methods(vec!["GET", "POST", "DELETE"])
    }

    // Allow any origin for public API

    fn cors_conf_any() -> Cors {
        Cors::default()
        .allow_any_origin()
        .send_wildcard()
        .allow_any_method()
        .allow_any_header()
    }

    // Allow Credentials
    
    fn cors_conf_cred() -> Cors {
        Cors::default()
        .allowed_origin("https://www.myapp.test")
        .supports_credentials()
        .allowed_methods(vec!["GET" , "POST"])
        .allowed_headers(vec![header::AUTHORIZATION , header::CONTENT_TYPE])
    }

    // Apply CORS to only one scope

    fn cors_conf_scope() -> Cors {
        Cors::default()
        .allowed_origin("https://www.myapp.test")
        .allowed_methods(vec!["GET", "POST"])
    }



    // however we can translating config into in this way 
    // speaking about the most optimal way to do it is this way

    #[derive(Debug , Clone , Deserialize)]
    #[serde(untagged)]
    enum OneOrMany {
        One(String) , 
        Many(Vec<String>),
    }

    #[derive(Debug , Clone , Deserialize)]
    struct CorsSetting {
        origins : Option<OneOrMany> ,
        methods : Option<OneOrMany> ,
        headers : Option<OneOrMany> ,
        #[serde(rename = "expose-headers")]
        expose_headers : Option<OneOrMany>,
        credentials : bool ,
        #[serde(rename = "send-wildcard")]
        send_wildcard : bool ,
        #[serde(rename = "max-age")]
        max_age : Option<usize> ,
        #[serde(rename = "block-on-origin-mismatch")]
        block_on_origin_mismatch: bool,
    }


    // and in following you can change it with functions
}