mod configs;

use {
    configs::{
        default_configs
    },
    actix_files::Files, actix_web::{
        App, HttpServer, guard, middleware:: {
            Logger
        }, web
    },
    env_logger::{
        Env
    }, 
    openssl::ssl::{
        SslAcceptor, SslFiletype, SslMethod
    }
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut builder = match SslAcceptor::mozilla_intermediate(SslMethod::tls()) {
        Ok(builder) => builder ,
        Err(err) => {
            err.errors().iter().for_each(|err| {
                eprintln!("Error : {}" , err);
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

    println!("Starting HTTPS server at https://www.myapp.test:433");

    HttpServer::new(move || {
        let logger = Logger::new("%s %a %{User-Agent}i");
        
        App::new().service({
            web::scope("")
            .wrap(logger)
            // Now lets load static_files into the server
            .service(Files::new("/static/html" , "./actix_practices/static/html"))
            .guard(guard::Host("www.myapp.test"))
            .configure(default_configs)
        })
    })
    .bind_openssl("127.0.0.1:433", builder)?
    .run()
    .await

}