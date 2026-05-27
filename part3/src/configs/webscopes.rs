use {
    actix_web::{HttpResponse, Responder, get, post, web},
    serde::{Deserialize, Serialize},
    std::sync::Mutex,
};

pub mod scopes {

    use actix_web::web::Data;

    use super::{Deserialize, HttpResponse, Mutex, Responder, Serialize, get, post, web};

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

    pub fn json_fn(cfg: &mut web::ServiceConfig, request_data: web::Data<AppState>) {
        cfg.app_data(request_data.clone())
            .service(echo_json)
            .service(get_json_data);
    }

    #[derive(Deserialize, Serialize, Clone)]
    struct BasicDataJson {
        name: String,
        req: String,
    }

    #[derive(Debug, Clone)]
    struct SavedRequest {
        user: String,
        request: String,
        name: String,
        req: String,
    }

    pub struct AppState {
        saved: Mutex<Vec<SavedRequest>>,
    }

    impl AppState {
        pub fn new() -> Data<AppState> {
            Data::new(Self {
                saved: Mutex::new(Vec::new()),
            })
        }
    }

    // This handler uses two extractors: Path and Json.
    #[post("/json/{user}/{request}")]
    async fn echo_json(
        path: web::Path<(String, String)>,
        json: web::Json<BasicDataJson>,
        capture: web::Data<AppState>,
    ) -> impl Responder {
        let (user, request) = path.into_inner();

        let saved_req = SavedRequest {
            user: user.clone(),
            request: request.clone(),
            name: json.name.clone(),
            req: json.req.clone(),
        };

        match capture.saved.lock() {
            Ok(mut vec) => vec.push(saved_req),
            Err(err) => {
                eprintln!("Mutex poisoned: {}", err);
                return HttpResponse::InternalServerError().body("Internal server error!");
            }
        };

        HttpResponse::Ok().body(format!(
            "user : {} with request [ {} ]\njson info : {} , {}",
            user, request, json.name, json.req
        ))
    }

    #[get("/json/getdata")]
    async fn get_json_data(request_data: web::Data<AppState>) -> impl Responder {
        let mut json_string_list = String::new();

        match request_data.saved.lock() {
            Ok(vec) => {
                vec.iter().for_each(|request| {
                    json_string_list.push_str(&format!(
                        "user {} , request : {} , name : {} , req : {}\n",
                        request.user, request.request, request.name, request.req
                    ));
                });
            }
            Err(err) => {
                eprintln!("Mutex poisoned: {}", err);
                return HttpResponse::InternalServerError().body("Internal server error!");
            }
        };

        HttpResponse::Ok().body(format!("Whole requsts that we got : {}", json_string_list))
    }

    pub fn query(cfg: &mut web::ServiceConfig) {
        cfg
        .service(query_index)
        .service(query_index_advanced);
    }

    #[get("/query")]
    async fn query_index(info: web::Query<Info>) -> String {
        format!("Welcome {} {}!", info.name, info.lastname)
    }

    #[derive(Deserialize)]
    pub struct Info {
        name: String,
        lastname: String,
    }

    #[derive(Deserialize)]
    pub struct AdvancedInfo {
        #[serde(rename = "advanced.name")]
        name: String,

        #[serde(rename = "advanced.lastname")]
        lastname: String,
    }

    #[get("/queryadvanced")]
    async fn query_index_advanced(info: web::Query<AdvancedInfo>) -> String {
        format!("Customer info {} {}!", info.name, info.lastname)
    }
}
