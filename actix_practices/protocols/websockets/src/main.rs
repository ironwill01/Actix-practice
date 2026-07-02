use {
    actix_web::{
        App, 
        HttpServer, 
        middleware::Logger, 
        web,
    }, 
    env_logger::Env, 
    openssl::ssl::{
        SslAcceptor, 
        SslFiletype, 
        SslMethod
    },
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
            err.errors().into_iter().for_each(|err| {
                eprintln!("Error : {}" , err)
            });
            panic!("Could not create SSL builder")
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
            web::scope("www.myapp.test")
        )
    })
    .bind_openssl("127.0.0.1:433", builder)?
    .run()
    .await
}