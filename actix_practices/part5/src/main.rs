mod configs;

use {
    crate::configs::{AppState, custom_resp},
    actix_web::{App, HttpServer, guard, middleware::Logger, web},
    configs::{UsersDataBase, components, defaults, errors, json_fn, query},
    openssl::ssl::{SslAcceptor, SslFiletype, SslMethod},
};

// This time its about Errors
// for more info just go to webscope library

// Error logging
#[rustfmt::skip]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let users_struct = UsersDataBase::new();

    let request_data = AppState::new();

    println!("Starting HTTPS server at https://www.myapp.test:443");

    let mut builder = match SslAcceptor::mozilla_intermediate(SslMethod::tls()) {
        Ok(ssl) => ssl,
        Err(errs) => {
            errs.errors().iter().for_each(|err| {
                println!("Error building TLS: {:?}", err.reason());
            });
            panic!("Error: could not create the SslAcceptor!");
        }
    };

    builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .expect("Error finding the key.pem file!");

    builder
        .set_certificate_chain_file("cert.pem")
        .expect("Error finding the cert.pem file!");

    builder
        .check_private_key()
        .expect("Error: private key does not match certificate!");

    // Wrapper around error logging
    unsafe {
        std::env::set_var("RUST_LOG", "info");
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    env_logger::init();

    HttpServer::new(move || {
        let user_data = users_struct.clone();
        let request_data = request_data.clone();

        let logger = Logger::default();

        App::new().service( 
            web::scope("")
                .guard(guard::Host("www.myapp.test"))
                .wrap(logger)
                .configure(defaults)
                .configure(move |cfg| {
                    components(cfg, user_data);
                })
                .configure(move |cfg| {
                    json_fn(cfg, request_data);
                })
                .configure(query)
                .configure(custom_resp)
                .configure(errors),
        )
    })
    .bind_openssl("127.0.0.1:443", builder)?
    .run()
    .await
}
