use {
    actix_web::{
        self, Either, HttpResponse, Responder, body::BoxBody, get, http::{header::ContentType , StatusCode}, post,
        web, web::Data,
    },
    derive_more::{self},
    futures::{future::ok, stream::once},
    serde::{self, Deserialize, Serialize},
    std::sync::Mutex,
};

pub mod scopes {

    use super::{
        BoxBody, ContentType, Data, Deserialize, Either, HttpResponse, Mutex, Responder, Serialize,
        actix_web::Error, derive_more::Display, derive_more::Error, get, ok, once, post, web, actix_web::error , StatusCode
    };

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

    // quesry extractors in rust
    pub fn query(cfg: &mut web::ServiceConfig) {
        cfg.service(query_index).service(query_index_advanced);
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

    // using serde you can make your var named in URL synrax
    // like this /queryadvanced?advanced.name=nikan&advanced.lastname=sadeghi
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

    // Now you can create your own handel
    // actix support built in types to be defined as impl Responder
    // it mean that you can create your own handel using normal types and maybe complex ones
    pub fn custom_resp(cfg: &mut web::ServiceConfig) {
        cfg.service(
            web::scope("/customs")
                .service(custom_obj)
                .service(stream)
                .service(either),
        );
    }

    #[derive(Serialize)]
    struct MyObj {
        name: &'static str,
    }

    //Responder
    impl Responder for MyObj {
        type Body = BoxBody;

        fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
            let body = match serde_json::to_string(&self) {
                Ok(string) => string,
                Err(err) => {
                    return HttpResponse::InternalServerError().body(format!("Error {}", err));
                }
            };

            // Create response and set content type
            HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(body)
        }
    }

    #[get("/custom_name")]
    async fn custom_obj() -> impl Responder {
        MyObj { name: "cool guy !" }
    }

    // There are many other other type of handelers
    // for example we want to create and streaming response
    // Response body can be generated asynchronously. In this case, body must implement the stream trait Stream<Item = Result<Bytes, Error>>, i.e.
    #[get("/streaming")]
    async fn stream() -> HttpResponse {
        let body = once(ok::<_, actix_web::Error>(web::Bytes::from_static(
            b"Binary in the app lol !",
        )));

        HttpResponse::Ok()
            .content_type(ContentType::json())
            .streaming(body)
    }

    // You can also have either response which work with right or left answer
    // you can write either as another type like this to make it less ... ugly
    // type RegisterResult = Either<HttpResponse, Result<&'static str, Error>>
    #[get("/either/{ok}")]
    async fn either(p: web::Path<bool>) -> Either<HttpResponse, Result<&'static str, Error>> {
        if *p {
            Either::Right(Ok("Hello !"))
        } else {
            Either::Left(HttpResponse::BadRequest().body("BadData !"))
        }
    }

    // Custom errors
    pub fn errors(cfg: &mut web::ServiceConfig) {
        cfg.service(web::scope("/err").service(error_test).service(complex_error_test).service(error_helper));
    }

    #[derive(Debug, Error, Display)]
    #[display("my error: {name}")]
    struct MyErr {
        name: &'static str,
    }

    impl error::ResponseError for MyErr {}

    #[get("/error_test/{answer}")]
    async fn error_test(p: web::Path<String>) -> Result<&'static str, MyErr> {
        if *p == "true" {
            Ok("true")
        } else {
            Err(MyErr {
                name: "Custom err : Bad req ",
            })
        }
    }

    // However you can still override the error_response in the ResponseError
    #[derive(Debug, Display, Error)]
    enum MyError {
        #[display("internal error")]
        InternalError,

        #[display("bad request")]
        BadClientData,

        #[display("timeout")]
        Timeout,
    }

    impl error::ResponseError for MyError {
        fn error_response(&self) -> HttpResponse<BoxBody> {
            HttpResponse::build(self.status_code())
                .insert_header(ContentType::html())
                .body(self.to_string())
        }

        fn status_code(&self) -> StatusCode {
            match *self {
                MyError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
                MyError::BadClientData => StatusCode::BAD_REQUEST,
                MyError::Timeout => StatusCode::GATEWAY_TIMEOUT,
            }
        }
    }

    #[get("/complex_error_test/{data}")]
    async fn complex_error_test(p: web::Path<String>) -> Result<&'static str, MyError> {
        if *p == "true" {
            Ok("true")
        } else {
            Err(MyError::BadClientData)
        }
    }


    // Actix Web provides a set of error helper functions that are useful for generating specific HTTP error codes from other errors. 
    //Here we convert SimpleErr, which doesn't implement the ResponseError trait, to a 400 (bad request) using map_err:

    #[derive(Debug)]
    struct SimpleErr {
        name : String,
    }


    // I made some change here just to let you put your message as error
    // it is pointless tho i still wanted work with the data myself
    #[get("/error_helper/{request}")]
    async fn error_helper(p : web::Path<String>) -> actix_web::Result<String> {
        let result = Err(SimpleErr { name : p.to_string()});
        result.map_err(|err| {
            error::ErrorBadRequest(format!("Error 400 : {}" , err.name))
        })
    }
}
