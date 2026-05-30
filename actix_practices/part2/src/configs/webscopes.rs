use {
    actix_web::{HttpResponse, Responder, web},
    std::sync::Mutex,
};

pub use scopes::{init, load_users, users_config};

pub mod scopes {
    use super::*;
    use actix_web::web::Data;

    pub fn init(cfg: &mut web::ServiceConfig) {
        cfg.service(
            web::resource("/")
                .app_data(web::Data::new(String::from("IW-WEB")))
                .route(web::get().to(welcome_page)),
        );
    }

    async fn welcome_page(app_name: web::Data<String>) -> impl Responder {
        format!("Welcome to {}", app_name.as_str())
    }

    pub fn users_config(cfg: &mut web::ServiceConfig, data: Data<UserData>) {
        cfg.app_data(data)
            .route("/users", web::get().to(users))
            .route("/postuser", web::post().to(post_users));
    }

    #[derive(Debug)]
    pub struct UserData {
        pub(crate) users: Mutex<Vec<String>>,
        pub(crate) counter: Mutex<i32>,
    }

    pub fn load_users() -> web::Data<UserData> {
        web::Data::new(UserData {
            users: Mutex::new(Vec::new()),
            counter: Mutex::new(0),
        })
    }

    async fn post_users(req_body: String, users_list: Data<UserData>) -> impl Responder {
        let _ = match users_list.users.lock() {
            Ok(mut vec) => vec.push(req_body.clone()),
            Err(err) => panic!("Mutex guard broken : {}", err),
        };
        HttpResponse::Ok().body(req_body)
    }

    async fn users(users_list: Data<UserData>) -> impl Responder {
        let mut lock_perm = match users_list.counter.lock() {
            Ok(num) => num,
            Err(err) => {
                panic!("Mutex guard broken : {}", err)
            }
        };

        let usernames = match users_list.users.lock() {
            Ok(users) => users.join(" "),
            Err(err) => {
                panic!("Mutex guard broken : {}", err)
            }
        };

        let _ = *lock_perm += 1;

        format!(
            "current users : {}\nthis page loaded {} time !",
            usernames, lock_perm
        )
    }
}
