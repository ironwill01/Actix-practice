mod configs;
use {
    actix_web::{
        App , HttpServer , 
        middleware::{Logger},
        web,
        guard
    },
    openssl::ssl::{SslAcceptor , SslFiletype , SslMethod},
    configs::{middleware_configure}
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    unsafe {
        std::env::set_var("RUST_LOG", "info");
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    let mut builder = match SslAcceptor::mozilla_intermediate(SslMethod::tls()) {
        Ok(builder) => {
            builder
        } ,
        Err(err) => {
            err.errors().iter().for_each(|err| {
                println!("Error : {}" , err);
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

    println!("Starting HTTPS server at https://www.myapp.test:433");

    env_logger::init();

    HttpServer::new(move || {
        let logger = Logger::default();
        App::new().service({
            web::scope("")
            .guard(guard::Host("www.myapp.test"))
            .configure(|cfg| {
                middleware_configure(cfg);
            })
        })
        .wrap(logger)
    })
    .bind_openssl("127.0.0.1:433", builder)?
    .run()
    .await
}