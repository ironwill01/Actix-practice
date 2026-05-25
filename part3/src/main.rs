mod configs;

use {
    actix_web::{guard, web, App, HttpServer},
    configs::{components, defaults, UsersDataBase},
    openssl::ssl::{SslAcceptor, SslFiletype, SslMethod},
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let users_struct = UsersDataBase::new();

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

    HttpServer::new(move || {
        let user_data = users_struct.clone();

        App::new().service(
            web::scope("")
                .guard(guard::Host("www.myapp.test"))
                .configure(defaults)
                .configure(move |cfg| {
                    components(cfg, user_data);
                }),
        )
    })
    .bind_openssl("127.0.0.1:443", builder)?
    .run()
    .await
}