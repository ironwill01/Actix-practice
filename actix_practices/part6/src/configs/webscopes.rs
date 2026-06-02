use {
    actix_web::{
        HttpResponse, Responder, get,
        post,
        web::{self},
    },
    derive_more::Display,
    serde::Deserialize,
    std::sync::Mutex,
};

pub mod scopes {
    use super::*;
    pub trait DataBase 
    where Self : Default
    {
        fn new() -> web::Data<Self> {
            web::Data::new(Self::default())
        }
    }

    pub fn simple_index(cfg: &mut web::ServiceConfig, users_and_request: web::Data<UserAppState> , userdatabase : web::Data<UserDataBase>) {
        cfg.service(
            web::scope("/simple")
                .app_data(users_and_request)
                .app_data(userdatabase)
                .service(set_users_json)
                .service(get_users_json)
                .service(setusers)
                .service(getusers)
        );
    }

    #[derive(Debug, Display, Default, Deserialize)]
    #[display("Requset with key value of : {key} : {value}")]
    struct JsonData {
        key: String,
        value: String,
    }

    #[derive(Debug, Display, Default, Deserialize)]
    #[display("Requset with key value of : {key} : {value} from user {user} : {password}")]
    struct JsonUserData {
        user: String,
        password: String,
        key: String,
        value: String,
    }

    #[derive(Default)]
    pub struct UserAppState {
        users: Mutex<Vec<JsonUserData>>,
    }

    impl DataBase for UserAppState {}

    #[post("/set_users_json/{user}/{password}")]
    async fn set_users_json(
        p: web::Path<(String, String)>,
        json: web::Json<JsonData>,
        capture: web::Data<UserAppState>,
    ) -> impl Responder {
        let _ = match capture.users.lock() {
            Ok(mut users) => {
                let path_binding = p.into_inner();
                let json = json.into_inner();
                users.push(JsonUserData {
                    user: path_binding.0,
                    password: path_binding.1,
                    key: json.key,
                    value: json.value,
                });
            }
            Err(err) => {
                eprintln!("Mutex guard poisoned : {}", err);
                return HttpResponse::InternalServerError().body("Internal server error!");
            }
        };

        HttpResponse::Ok().body("User added !")
    }

    #[get("/get_users_json")]
    async fn get_users_json(data: web::Data<UserAppState>) -> impl Responder {
        let _ = match data.users.lock() {
            Ok(users) => {
                let body = users
                    .iter()
                    .map(|user| user.to_string())
                    .collect::<Vec<_>>()
                    .join("\n");
                return HttpResponse::Ok().body(body);
            }
            Err(err) => {
                eprintln!("Mutex guard poisoned : {}", err);
                return HttpResponse::InternalServerError().body("Internal server error!");
            }
        };
    }

    #[derive(Deserialize , Default)]
    struct UserInfo {
        #[serde(rename = "User.name")]
        name: String,
        #[serde(rename = "User.pass")]
        password: String,
    }

    #[derive(Deserialize , Default)]
    pub struct UserDataBase {
        users: Mutex<Vec<UserInfo>>,
    }

    impl DataBase for UserDataBase {}

    #[get("/set_user")]
    async fn setusers(
        query: web::Query<UserInfo>,
        data_base: web::Data<UserDataBase>,
    ) -> impl Responder {
        match data_base.users.lock() {
            Ok(mut users) => users.push(UserInfo {
                name: query.name.clone(),
                password: query.password.clone(),
            }),
            Err(err) => {
                eprintln!("Mutex guard poisoned : {}", err);
                return HttpResponse::InternalServerError().body("Internal server error!");
            }
        }

        HttpResponse::Ok().body("User added to the database !")
    }

    #[get("/get_users")]
    async fn getusers(data_base: web::Data<UserDataBase>) -> impl Responder {
        let mut body = String::new();
        match data_base.users.lock() {
            Ok(users) => {
                HttpResponse::Ok().body(users.iter().for_each(|users| {
                    body.push_str(&format!(
                        "User : {} with password : {}\n",
                        users.name, users.password
                    ));
                }));
            }
            Err(err) => {
                eprintln!("Mutex guard poisoned : {}", err);
                return HttpResponse::InternalServerError().body("Internal server error!");
            }
        }
        HttpResponse::Ok().body(body)
    }
}
