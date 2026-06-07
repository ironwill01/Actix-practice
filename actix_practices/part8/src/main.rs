pub mod configs;

use {
    actix_web::{App, HttpServer, guard, middleware::{self ,Logger}, web},
    configs::{custom_configs, default_config, not_found},
    openssl::ssl::{SslAcceptor, SslFiletype, SslMethod},
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    unsafe {
        std::env::set_var("RUST_LOG", "info");
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    env_logger::init();

    let mut ssl =
        SslAcceptor::mozilla_intermediate(SslMethod::tls()).expect("Failed to create TLS acceptor");

    ssl
    .set_private_key_file("key.pem", SslFiletype::PEM)
        .expect("Failed to load private key");

    ssl
    .set_certificate_chain_file("cert.pem")
        .expect("Failed to load certificate chain");

    println!("Starting HTTPS server at https://www.myapp.test:433");

    HttpServer::new(|| {
        let logger = Logger::default();
        App::new()
            .service(
                web::scope("")
                    .guard(guard::Host("www.myapp.test"))
                    .configure(default_config)
                    .configure(|cfg| {
                        custom_configs(cfg);
                    }),
            )
            .wrap(logger)
            // Here you can have compress for your data from an response
            .wrap(middleware::Compress::default())
            // You can change default services like 404 in here
            .default_service(web::route().to(not_found))
    })
    .bind_openssl("127.0.0.1:443", ssl)?
    .run()
    .await
}
