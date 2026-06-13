mod configs;
use {
   actix_web::{App, HttpServer, guard, middleware::Logger, web}, 
   configs::{configure_middleware_addone_wrapped, configure_middleware_addone_wrapped_fn, configure_middleware_wrapped, middleware_configure , JsonAppState , json_configs}, 
   env_logger::{self, Env}, openssl::ssl::{SslAcceptor, SslFiletype, SslMethod}
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    unsafe {
        std::env::set_var("RUST_LOG", "info");
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    let mut builder = match SslAcceptor::mozilla_intermediate(SslMethod::tls()) {
        Ok(builder) => builder,
        Err(err) => {
            err.errors().iter().for_each(|err| {
                println!("Error : {}", err);
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

    // We can make an env 
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    let json_state = JsonAppState::new();
    HttpServer::new(move || {
        
        let logger = Logger::default();
        let custom_logger = Logger::new("%s %a %{User-Agent}i");
        App::new()
            .service({
                web::scope("")
                    .app_data(json_state.clone())
                    .guard(guard::Host("www.myapp.test"))
                    .configure(|cfg| {
                        //middleware_configure(cfg);
                        //configure_middleware_wrapped(cfg);
                        //configure_middleware_addone_wrapped(cfg);
                        //configure_middleware_addone_wrapped_fn(cfg);
                    }).configure(|cfg | { 
                        json_configs(cfg , json_state.clone())
                    })
            })
            .wrap(logger)
            .wrap(custom_logger)
    })
    .bind_openssl("127.0.0.1:433", builder)?
    .run()
    .await
}
