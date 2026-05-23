pub mod configs;

use actix_web::{App, HttpServer, Responder, guard, web};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

use configs::webscopes::{init, load_users, users_config};

// whole point of part 2 is basically get to know the server struct and 
// TLS logic in actix


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Loading server ...");

    let users = load_users();

    // OpenSSL for TSL connection
    let mut builder = match SslAcceptor::mozilla_intermediate(SslMethod::tls()) {
        Ok(ssl) => ssl,
        Err(err) => {
            for error in err.errors() {
                println!(
                    "Error could not establish the TSL : {}",
                    error.reason().unwrap()
                )
            }
            panic!("TSL auth failed ...")
        }
    };

    // Build the cert part
    builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .expect("Could not load key.pem");

    builder
        .set_certificate_chain_file("cert.pem")
        .expect("Could not load cert.pem");

    builder
        .check_private_key()
        .expect("key.pem does not match cert.pem");

    // You can make your server multi-threaded using methods
    HttpServer::new(move || {
        App::new().service(
            web::scope("")
                .guard(guard::Host("www.myapp.test"))
                .configure(init)
                .configure({
                    let users = users.clone();

                    move |cfg| {
                        users_config(cfg, users.clone());
                    }
                }),
        )
    })
    .workers(4)
    .bind_openssl("127.0.0.1:8443", builder)?
    .run()
    .await
}
