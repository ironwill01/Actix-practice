use {
    super::TEMPLATE_PATH,
    super::UserMessage,
    actix_web::{
        HttpResponse , HttpRequest , Result
        , get 
        , post
        , web::{
            self
        }
    },
    actix_files::{
        NamedFile
    },
    serde::{
        Serialize , 
        Deserialize
    },
    std::path::{
        PathBuf
    },
    urlencoding::{self},
    tokio::{
        sync::Mutex
    }
};

#[derive(Debug , Serialize , Deserialize)]
struct SignForm {
    name : String , 
    message : String
}

pub fn signpage_config(cfg : &mut web::ServiceConfig , data : web::Data<Mutex<Vec<UserMessage>>>) {
    cfg
    .app_data(data)
    .service(sign_message_page)
    .service(signup_html_page)
    .service(signup_page)
    .service(sign_message_post)
    .service(success_page);
}

async fn render_signup_page(req : HttpRequest) -> Result<HttpResponse> {
    let path = PathBuf::from(format!("{}/signup.html" , TEMPLATE_PATH));
    
    let response = match NamedFile::open(path) {
        Ok(page) => page.into_response(&req),
        Err(err) => {
            eprintln!("Error : {}" , err);
            return Ok(HttpResponse::InternalServerError().body("<h1>Error code 500</h1>"));
        }
    };

    Ok(response)
}

#[get("/sign_message")]
async fn sign_message_page(req : HttpRequest) -> Result<HttpResponse> {
    render_signup_page(req).await
}

#[get("/signup.html")]
async fn signup_html_page(req : HttpRequest) -> Result<HttpResponse> {
    render_signup_page(req).await
}

#[get("/signup")]
async fn signup_page(req : HttpRequest) -> Result<HttpResponse> {
    render_signup_page(req).await
}

#[post("/sign")]
async fn sign_message_post(form : web::Form<SignForm> , data : web::Data<Mutex<Vec<UserMessage>>>) -> Result<HttpResponse> {
    println!("{} : {}" , form.name , form.message);

    let redirect_url = format!(
            "/success?name={}&message={}",
            urlencoding::encode(&form.name),
            urlencoding::encode(&form.message)
        );

    let _ = data.lock().await.push(
        UserMessage::from(form.name.clone(), form.message.clone())
    );

    Ok(HttpResponse::SeeOther()
        .append_header(("Location", redirect_url.as_str()))
        .finish())
}

#[get("/success")]
async fn success_page(req: HttpRequest) -> Result<HttpResponse> {
    let path = PathBuf::from(format!("{}/success.html", TEMPLATE_PATH));
    
    match NamedFile::open(path) {
        Ok(file) => Ok(file.into_response(&req)),
        Err(_) => Ok(HttpResponse::NotFound().body("Success page not found")),
    }
}