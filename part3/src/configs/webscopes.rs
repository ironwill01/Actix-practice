use {
    actix_web::{HttpResponse, Responder, web},
    std::sync::Mutex,
};

pub mod scopes {
    use super::*;

    pub fn defaults(cfg: &mut web::ServiceConfig) {
        cfg.service(
            web::resource("/")
                .app_data(web::Data::new(String::from("IW-WEB")))
                .route(web::get().to(manual)),
        );
    }

    async fn manual(app_name: web::Data<String>) -> impl Responder {
        format!("Welcome to the {} !", app_name.as_str())
    }

    pub fn components(cfg: &mut web::ServiceConfig, data: web::Data<UsersDataBase>) {
        cfg.app_data(data.clone())
            .route("/postuser", web::post().to(post_users))
            .route("/users", web::get().to(users));
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

    async fn post_users(req: String, users: web::Data<UsersDataBase>) -> impl Responder {
        let mut users_lock = match users.users.lock() {
            Ok(users_vec) => users_vec,
            Err(err) => {
                eprintln!("Mutex poisoned: {}", err);
                return HttpResponse::InternalServerError().body("Internal server error!");
            }
        };

        users_lock.push(req);

        HttpResponse::Ok().body("User added")
    }

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
}
