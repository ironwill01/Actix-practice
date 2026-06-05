use {
    actix_web::{
        HttpRequest, HttpResponse, Responder, Result,
        body::BoxBody,
        get, guard,post,
        http::header::{self, ContentType},
        web::{self},
    },
    derive_more::Display,
    serde::{Deserialize , Serialize},
    std::sync::Mutex,
};

pub mod scopes {

use super::*;
    pub trait DataBase
    where
        Self: Default,
    {
        fn new() -> web::Data<Self> {
            web::Data::new(Self::default())
        }
    }

    pub fn simple_index(
        cfg: &mut web::ServiceConfig,
        users_and_request: web::Data<UserAppState>,
        userdatabase: web::Data<UserDataBase>,
    ) {
        cfg.service(
            web::scope("/simple")
                .app_data(users_and_request)
                .app_data(userdatabase)
                .service(set_users_json)
                .service(get_users_json)
                .service(setusers)
                .service(getusers)
                .service(index)
                // this one is actually for URL dispatch using it for `url_for`
                .service(url_index)
                .service(
                    web::resource("/get_users/{a}/{b}/")
                        .name("blud")
                        .guard(guard::Get())
                        .to(HttpResponse::Ok),
                ),
        );
    }

    #[derive(Debug, Display, Default, Deserialize)]
    #[display("Requset with key value of : {key} : {value}")]
    struct JsonData {
        key: String,
        value: String,
    }

    #[derive(Debug, Display, Default, Deserialize , Serialize)]
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

    #[post("/user_json/{user}/{password}")]
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

    #[get("/user_json")]
    async fn get_users_json(data: web::Data<UserAppState>) -> impl Responder {
        let _ = match data.users.lock() {
            Ok(users) => {
                return HttpResponse::Ok()
                .insert_header(("content-type" , "application/json"))
                .json(&*users);
            }
            Err(err) => {
                eprintln!("Mutex guard poisoned : {}", err);
                return HttpResponse::InternalServerError().body("Internal server error!");
            }
        };
    }

    #[derive(Deserialize, Default)]
    struct UserInfo {
        #[serde(rename = "User.name")]
        name: String,
        #[serde(rename = "User.pass")]
        password: String,
    }

    #[derive(Deserialize, Default)]
    pub struct UserDataBase {
        users: Mutex<Vec<UserInfo>>,
    }

    impl DataBase for UserDataBase {}

    // i still found no use here even tho i impl respond_to there is no point ( literally )
    impl Responder for UserDataBase {
        type Body = BoxBody;
        fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
            match self.users.lock() {
                Ok(users) => {
                    let body = users
                        .iter()
                        .map(|u| format!("{} : {}", u.name, u.password))
                        .collect::<Vec<_>>()
                        .join("\n");

                    HttpResponse::Ok()
                        .insert_header(ContentType::html())
                        .body(body)
                }
                Err(err) => {
                    eprintln!("Mutex poisoned : {}", err);
                    HttpResponse::InternalServerError().body("Internal server error !")
                }
            }
        }
    }

    // Also for URL dispatch ( which we didnt do here ) you need to either
    // use match_info from HttpRequset or make an struct and use it with path
    // which you need to use Deserialize from serde so you can actually do it
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

    #[get("/get_users/{name}/{lastname}")]
    async fn getusers(data_base: web::Data<UserDataBase> , path : web::Path<(String , String)>) -> impl Responder {
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
        let (firstname , lastname) = path.into_inner();
        HttpResponse::Ok().body(format!("hey {} {}\n{}" , firstname , lastname , body))
    }

    // Url syntax and the how HttpRequset can extract information from URL using `match_info()`
    #[get("/a/{v1}/{v2}/")]
    async fn index(req: HttpRequest) -> Result<String> {
        let v1: u8 = req.match_info().get("v1").unwrap().parse().unwrap();
        let v2: u8 = req.match_info().query("v2").parse().unwrap();
        let (v3, v4): (u8, u8) = req.match_info().load().unwrap();
        Ok(format!("Values {} {} {} {}", v1, v2, v3, v4))
    }

    #[get("/test/")]
    async fn url_index(req: HttpRequest) -> Result<HttpResponse> {
        let url = req.url_for("blud", ["nikan", "sadeghi"])?;
        Ok(HttpResponse::Found()
            .insert_header((header::LOCATION, url.as_str()))
            .finish())
    }

    // Overall i found that most of the stuff we do about URL dispatch
    // is already done in last chapters
}
