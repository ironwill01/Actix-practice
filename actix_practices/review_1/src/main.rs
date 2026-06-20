mod configs;
use {
    actix_files::Files, 
    actix_web::{
        App, HttpServer, middleware, web
    },
    openssl::ssl::{
        SslAcceptor, SslFiletype, SslMethod
    },
    env_logger::{
        Env
    },
    configs::{
        default_configs
    }
};


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    unsafe {
        std::env::set_var("RUST_LOG", "info");
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    let mut builder = match SslAcceptor::mozilla_intermediate(SslMethod::tls()) {
        Ok(builder) => builder ,
        Err(err) => {
            err.errors().iter().for_each(|err| {
                eprintln!("Error : {}" , err)
            });
            panic!()
        }
    };

    builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .expect("Failed to load private key");

    builder
        .set_certificate_chain_file("cert.pem")
        .expect("Failed to load certificate chain");

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    println!("Starting HTTPS server at https://www.myapp.test:443");
    
    HttpServer::new(move || {
        let logger = middleware::Logger::new("%s %a %{User-Agent}i");

        App::new()
        .service(Files::new("static", "./actix_practices/review_1/src/templates").show_files_listing())
        .service(
            web::scope("")
            .configure(default_configs)
        ).wrap(logger)
    })
    .bind_openssl("127.0.0.1:443", builder)?
    .run()
    .await
}
