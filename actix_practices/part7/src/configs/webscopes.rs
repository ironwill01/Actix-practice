use {
    actix_web::{
        HttpResponse, Responder,
        body::BoxBody,
        get, post,
        http::header::{ContentType},
        web::{self},
        error,
        Error
    },
    derive_more::Display,
    serde::{Deserialize , Serialize},
    std::sync::Mutex,
    futures::StreamExt,
};


// Dont pay attention we gotta use this real soon for payload 
// we gonna load data to memory in chunks ( can be useful for loading video or any type of data )
const MAX_SIZE : usize = 262_144;

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
        );
    }

    // JSON type data we work with this in this part 
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

    #[derive(Default , Deserialize , Serialize)]
    pub struct UserAppState {
        users: Mutex<Vec<JsonUserData>>,
    }

    impl DataBase for UserAppState {}


    // as you can see we extract JsonData using web::Json
    // you can also manually load the payload into memory and then desreialize it.
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

    // This function basically load the data in memory in chunks with the size we set for it
    // and then start to make json out of the struct itself if it derive from serde_json traits 
    // also for content encoding
    // actix Web automatically decompresses payloads. The following codecs are supported:
    // Brotli Gzip Deflate Zstd
    #[get("/user_json/manual")]
    async fn get_users_json_manual(mut data : web::Payload) -> Result<HttpResponse, Error> {
        let mut body = web::BytesMut::new();
        while let Some(chunk) = data.next().await {
            let chuck = chunk?;

            // limit the max size of in-memory payload
            if(body.len() + chuck.len()) > MAX_SIZE {
                return Err(error::ErrorBadRequest("Payload overflow"));
            }
            body.extend_from_slice(&chuck);
        }

        let object = serde_json::from_slice::<UserAppState>(&body)?;
        Ok(HttpResponse::Ok().json(object))
    }

    // Query type data skip for this part let it stay here anyway
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

}
