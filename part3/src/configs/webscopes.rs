use {
    actix_web::{HttpResponse, Responder, get, post, web},
    serde::Deserialize,
    std::sync::Mutex,
};

pub mod scopes {
    use super::{Deserialize, HttpResponse, Mutex, Responder, get, post, web};

    pub fn defaults(cfg: &mut web::ServiceConfig) {
        cfg.app_data(web::Data::new(String::from("IW-WEB")))
            .service(manual);
    }

    // web::Data<String> is an extractor.
    // Extractors read data from the incoming request/app state.
    #[get("/")]
    async fn manual(app_name: web::Data<String>) -> impl Responder {
        format!("Welcome to the {} !", app_name.as_str())
    }

    pub fn components(cfg: &mut web::ServiceConfig, data: web::Data<UsersDataBase>) {
        cfg.app_data(data.clone())
            .service(post_users)
            .service(users);
    }

    #[derive(Debug)]
    pub struct UsersDataBase {
        pub(crate) users: Mutex<Vec<String>>,
        pub(crate) counter: Mutex<i32>,
    }

    impl UsersDataBase {
        pub fn new() -> web::Data<UsersDataBase> {
            web::Data::new(UsersDataBase {
                users: Mutex::new(Vec::new()),
                counter: Mutex::new(0),
            })
        }
    }

    #[post("/postuser")]
    async fn post_users(req: String, other_users: web::Data<UsersDataBase>) -> impl Responder {
        let mut users_lock = match other_users.users.lock() {
            Ok(users_vec) => users_vec,
            Err(err) => {
                eprintln!("Mutex poisoned: {}", err);
                return HttpResponse::InternalServerError().body("Internal server error!");
            }
        };

        users_lock.push(req);

        HttpResponse::Ok().body("User added")
    }

    #[get("/users")]
    async fn users(data_base: web::Data<UsersDataBase>) -> impl Responder {
        let mut counter_lock = match data_base.counter.lock() {
            Ok(num) => num,
            Err(err) => {
                eprintln!("Mutex poisoned: {}", err);
                return HttpResponse::InternalServerError().body("Internal server error!");
            }
        };

        *counter_lock += 1;

        let users_string = match data_base.users.lock() {
            Ok(user_vec) => user_vec.join(" , "),
            Err(err) => {
                eprintln!("Mutex poisoned: {}", err);
                return HttpResponse::InternalServerError().body("Internal server error!");
            }
        };

        HttpResponse::Ok().body(format!(
            "[{}]\nthis page loaded {} time!",
            users_string, *counter_lock
        ))
    }

    pub fn json_fn(cfg: &mut web::ServiceConfig) {
        cfg.service(echo_json);
    }

    #[derive(Deserialize, Debug)]
    struct BasicData {
        name: String,
        req: String,
    }

    // This handler uses two extractors: Path and Json.
    #[post("/json/{user}/{request}")]
    async fn echo_json(
        path: web::Path<(String, String)>,
        json: web::Json<BasicData>,
    ) -> impl Responder {
        let path: (String, String) = path.into_inner();

        format!(
            "user : {} with request [ {} ]\njson info : {} , {}",
            path.0, path.1, json.name, json.req
        )
    }
}
