mod configs;

use {
    actix_web::{
        App , HttpServer ,
        web ,
        guard ,
        middleware::{
            Logger
        }
    },
    openssl::ssl::{
        SslAcceptor 
        , SslFiletype 
        , SslMethod
    },
    env_logger::{Env},
    configs::{
        default_config
    }
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut builder = match SslAcceptor::mozilla_intermediate(SslMethod::tls()) {
        Ok(builder) => builder ,
        Err(errors) => {
            for err in errors.errors() {
                eprintln!("Error : {}" , err);
            }
            panic!();
        }
    };

    builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .expect("Failed to load private key");

    builder
        .set_certificate_chain_file("cert.pem")
        .expect("Failed to load certificate chain");

    println!("Starting HTTPS server at https://www.myapp.test:433");

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    HttpServer::new( move || {
        let logger = Logger::new("%s %a %{User-Agent}i");

        App::new().service(
            web::scope("")
            .guard(guard::Host("www.myapp.test"))
            .configure(default_config)
        ).wrap(logger)
    })
    .bind_openssl("127.0.0.1:433", builder)?
    .run()
    .await
}