pub mod configs;
use {
    actix_web::{
        App, HttpServer,
        guard::{self},
        middleware::Logger,
        web::{self},
    }, configs::{DataBase, UserAppState, simple_index , UserDataBase}, openssl::ssl::{SslAcceptor, SslFiletype, SslMethod}
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let users_request = UserAppState::new();
    let user_database = UserDataBase::new();

    // load SSL verification
    let mut builer = match SslAcceptor::mozilla_intermediate_v5(SslMethod::tls()) {
        Ok(builder) => builder,
        Err(err) => {
            panic!("TLS builder for auth is failed : {}", err)
        }
    };

    builer
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .expect("Error could not find the key file for TLS !");

    builer
        .set_certificate_chain_file("cert.pem")
        .expect("Error could not find the certification file !");

    unsafe {
        std::env::set_var("RUST_LOG", "info");
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    env_logger::init();

    println!("Starting HTTPS server at https://www.myapp.test:443");
    HttpServer::new(move || {
        let logger = Logger::default();
        App::new().service(
            web::scope("")
                .wrap(logger)
                .guard(guard::Host("www.myapp.test"))
                .configure(|cfg| {
                    simple_index(cfg, users_request.clone() , user_database.clone());
                }),
        )
    })
    .bind_openssl("127.0.0.1:443", builer)?
    .run()
    .await
}
