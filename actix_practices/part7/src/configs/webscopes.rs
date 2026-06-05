use {
    actix_web::{
        Error, HttpResponse, Responder, error, get, post,
        web::{self},
    },
    derive_more::Display,
    futures::StreamExt,
    serde::{Deserialize, Serialize},
    std::sync::Mutex,
};

// Dont pay attention we gotta use this real soon for payload
// we gonna load data to memory in chunks ( can be useful for loading video or any type of data )
const MAX_SIZE: usize = 262_144;

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

    pub fn simple_index(cfg: &mut web::ServiceConfig, users_and_request: web::Data<UserAppState>) {
        cfg.service(
            web::scope("/simple")
                .app_data(users_and_request)
                .service(set_users_json)
                .service(get_users_json)
                .service(formpost),
        );
    }

    // JSON type data we work with this in this part
    #[derive(Debug, Display, Default, Deserialize)]
    #[display("Requset with key value of : {key} : {value}")]
    struct JsonData {
        key: String,
        value: String,
    }

    #[derive(Debug, Display, Default, Deserialize, Serialize)]
    #[display("Requset with key value of : {key} : {value} from user {user} : {password}")]
    struct JsonUserData {
        user: String,
        password: String,
        key: String,
        value: String,
    }

    #[derive(Default, Deserialize, Serialize)]
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
                    .insert_header(("content-type", "application/json"))
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
    // One more note about actix and chunking
    // actix can automatically decodes chunked data using web::Payload as we used here
    async fn get_users_json_manual(mut data: web::Payload) -> Result<HttpResponse, Error> {
        let mut body = web::BytesMut::new();
        while let Some(chunk) = data.next().await {
            let chuck = chunk?;

            // limit the max size of in-memory payload
            if (body.len() + chuck.len()) > MAX_SIZE {
                return Err(error::ErrorBadRequest("Payload overflow"));
            }
            body.extend_from_slice(&chuck);
        }

        let object = serde_json::from_slice::<UserAppState>(&body)?;
        Ok(HttpResponse::Ok().json(object))
    }

    // Actix is also provides multipart stream support with an external crate, called actix-multipart
    // https://crates.io/crates/actix-multipart you can find more examples there

    // Created this function to show form data method but didnt had enough data to work with
    // Actix Web provides support for application/x-www-form-urlencoded encoded bodies with
    // the web::Form extractor which resolves to the
    // deserialized instance. The type of the instance must implement the Deserialize trait from serde.

    // also The UrlEncoded future can resolve into an error in several cases :
    // content type is not application/x-www-form-urlencoded
    // transfer encoding is chunked.
    // content-length is greater than 256k
    // payload terminates with error.

    #[derive(Deserialize)]
    struct FormUser {
        user: String,
    }

    #[post("/formpost")]
    async fn formpost(
        form: web::Form<FormUser>,
        database: web::Data<UserAppState>,
    ) -> HttpResponse {
        match database.users.lock() {
            Ok(users) => {
                if let Some(user) = users.iter().find(|u| u.user == form.user) {
                    HttpResponse::Ok().json(user)
                } else {
                    HttpResponse::NotFound().body(format!("User '{}' not found", form.user))
                }
            }
            Err(err) => {
                eprintln!("Mutex guard poisoned: {}", err);
                HttpResponse::InternalServerError().body("Internal server error!")
            }
        }
    }


    // This will stream data loads using Payload !
    #[get("/streampayload")]
    async fn bigload(mut body : web::Payload) -> Result<HttpResponse , Error> {
        let mut bytes = web::BytesMut::new();
        while let Some(data) = body.next().await {
            let data = data?;
            println!("Data : {:?}" , &data);
            bytes.extend_from_slice(&data);
        }
        Ok(HttpResponse::Ok().finish())
    }
}
