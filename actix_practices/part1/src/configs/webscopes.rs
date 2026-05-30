use actix_web::{HttpResponse, Responder, web , guard};

// i made whole of this thing in config mode
// basically parted it into different sections
// now you can follow it easier
pub mod scopes {

    use super::*;

    pub fn init_config(cfg: &mut web::ServiceConfig) {
        cfg.service(
            web::resource("/")
                .guard(guard::Host("www.myapp.test"))
                .route(web::get().to(|| async { HttpResponse::Ok().body("Hello from website!") })),
        );
    }

    struct UserBase {
        users: Vec<String>,
    }

    async fn hellohtml() -> impl Responder {
        "you called hello HTML"
    }

    async fn users(users: web::Data<UserBase>) -> String {
        users.users.join(" ")
    }

    pub fn static_scope(cfg: &mut web::ServiceConfig) {
        // this is how you create services and routes to an scope
        // prefix with webscope and then routing using route.
        // Application state is shared with all routes and resources within the same scope.
        cfg.service(
            web::scope("/static")
                .app_data(web::Data::new(UserBase {
                    users: vec![
                        "Nikan".to_string(),
                        "Parsa".to_string(),
                        "Shahab".to_string(),
                    ],
                }))
                .route("/hello", web::get().to(hellohtml))
                .route("/users", web::get().to(users)),
        );
    }

    async fn echo(req_body: String) -> impl Responder {
        HttpResponse::Ok().body(req_body)
    }

    async fn manual_hello() -> impl Responder {
        HttpResponse::Ok().body("Hey there")
    }

    pub fn dynamic_scope(cfg: &mut web::ServiceConfig) {
        cfg.service(
            web::scope("/dynamic")
                .route("/echo", web::post().to(echo))
                .route("/manual", web::get().to(manual_hello)),
        );
    }
}
