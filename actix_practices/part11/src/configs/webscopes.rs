use {
    actix_cors::Cors,
    actix_web::{
        Error, 
        HttpRequest, 
        HttpResponse, 
        Result, 
        get, 
        guard,
        http::{
            StatusCode,
            header::{
                self, 
                ContentType
            },
        },
        post, web,
    },
    serde::{
        Deserialize, 
        Serialize
    },
};

pub mod scopes {
    use super::*;

    pub fn default_config(cfg: &mut web::ServiceConfig) {
        cfg.service(homepage).service(home_redirect);
    }

    // never knew you can name handles
    #[get("/homepage", name = "homepage")]
    async fn homepage() -> Result<HttpResponse> {
        Ok(HttpResponse::build(StatusCode::OK)
            .insert_header(ContentType::html())
            .body("<h1>Hello to IW-APP</h1>"))
    }

    #[get("/")]
    async fn home_redirect(req: HttpRequest) -> Result<HttpResponse> {
        let url = req.url_for("homepage", [""])?; // for some reason i decided to generate URL for default page now
        println!("Redirecting to homepage ...");
        Ok(HttpResponse::Found()
            .insert_header((header::LOCATION, url.as_str()))
            .finish())
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
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![header::AUTHORIZATION, header::CONTENT_TYPE])
    }

    // Apply CORS to only one scope

    fn cors_conf_scope() -> Cors {
        Cors::default()
            .allowed_origin("https://www.myapp.test")
            .allowed_methods(vec!["GET", "POST"])
    }

    // however we can translating config into in this way
    // speaking about the most optimal way to do it is this way

    #[derive(Debug, Clone, Deserialize)]
    #[serde(untagged)]
    enum OneOrMany {
        One(String),
        Many(Vec<String>),
    }

    #[derive(Debug, Clone, Deserialize)]
    struct CorsSettings {
        origins: Option<OneOrMany>,
        methods: Option<OneOrMany>,
        headers: Option<OneOrMany>,
        #[serde(rename = "expose-headers")]
        expose_headers: Option<OneOrMany>,
        credentials: bool,
        #[serde(rename = "send-wildcard")]
        send_wildcard: bool,
        #[serde(rename = "max-age")]
        max_age: Option<usize>,
        #[serde(rename = "block-on-origin-mismatch")]
        block_on_origin_mismatch: bool,
    }

    // and in following you can change it with functions

    // Mapping rule : 
    //  origins: "*" maps to allow_any_origin(), not allowed_origin("*").
    //  methods: "*" maps to allow_any_method().
    //  headers: "*" maps to allow_any_header().
    //  A single origin like https://app.example.com maps to one allowed_origin(...) call.
    //  A list of origins maps to repeated allowed_origin(...) calls.
    //  A single method or header can be passed as a one-item iterator, or handled with the singular builder methods.

        

    fn cors_from_settings(settings: &CorsSettings) -> Cors {
        let mut cors = Cors::default();

        match settings.origins.as_ref() {
            None => {}
            Some(OneOrMany::One(origin)) if origin == "*" => {
                cors = cors.allow_any_origin();
            }
            Some(OneOrMany::One(origin)) => {
                cors = cors.allowed_origin(origin.as_str());
            }
            Some(OneOrMany::Many(origins)) => {
                for origin in origins {
                    cors = cors.allowed_origin(origin.as_str());
                }
            }
        }

        match settings.methods.as_ref() {
            None => {}
            Some(OneOrMany::One(method)) if method == "*" => {
                cors = cors.allow_any_method();
            }
            Some(OneOrMany::One(method)) => {
                cors = cors.allowed_methods([method.as_str()]);
            }
            Some(OneOrMany::Many(methods)) => {
                cors = cors.allowed_methods(methods.iter().map(String::as_str));
            }
        }

        match settings.headers.as_ref() {
            None => {}
            Some(OneOrMany::One(header)) if header == "*" => {
                cors = cors.allow_any_header();
            }
            Some(OneOrMany::One(header)) => {
                cors = cors.allowed_header(header.as_str());
            }
            Some(OneOrMany::Many(headers)) => {
                cors = cors.allowed_headers(headers.iter().map(String::as_str));
            }
        }

        match settings.expose_headers.as_ref() {
            None => {}
            Some(OneOrMany::One(header)) if header == "*" => {
                cors = cors.expose_any_header();
            }
            Some(OneOrMany::One(header)) => {
                cors = cors.expose_headers([header.as_str()]);
            }
            Some(OneOrMany::Many(headers)) => {
                cors = cors.expose_headers(headers.iter().map(String::as_str));
            }
        }

        if settings.credentials {
            cors = cors.supports_credentials();
        }

        if settings.send_wildcard {
            cors = cors.send_wildcard();
        }

        if let Some(max_age) = settings.max_age {
            cors = cors.max_age(max_age);
        }

        cors.block_on_origin_mismatch(settings.block_on_origin_mismatch)

    }

    // Wildcards, Credentials, and Caches

    // allow_any_origin() accepts any origin.

    // send_wildcard() changes the response header from echoing the request origin to sending Access-Control-Allow-Origin: *.
    
    // That distinction matters because credentials and wildcard responses cannot be combined :
    pub fn cors_conf_fail() -> Cors {
        Cors::default()
        .allow_any_origin()
        .supports_credentials()
        .send_wildcard()
    }

    // That configuration fails during startup. If your browser clients need cookies or authorization headers, prefer an explicit origin allowlist instead of *.

    // actix-cors also enables the Vary header by default. Keep that default unless you fully understand the caching implications. 
    // It tells CDNs and proxies that the CORS response can change based on request headers.

    // When To Use allowed_origin_fn
    // Most applications should keep CORS static and startup-configured. 
    // Use allowed_origin_fn only when your allowlist really must depend on request data or pattern matching, such as tenant subdomains :
    fn cors_conf_dynamic() -> Cors {
        Cors::default()
        .allowed_origin_fn(|origin , _req_header| {
            origin.as_bytes().ends_with(b".myapp.test")
        })
    }
    
    // That is different from loading a normal allowlist from config. 
    // If your configuration is just "one value, many values, or *", prefer the static builder methods shown above.

    // Applying CORS To Your App
    // Once the builder has been created from config, wrap it like any other middleware

}