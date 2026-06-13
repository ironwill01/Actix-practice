mod configs;
use {
   actix_session::{
        SessionMiddleware,
        storage::CookieSessionStore,
    }, 
    actix_web::{ 
        App, HttpServer, cookie::Key, guard, middleware::Logger, web
    }, 
    configs::{
        JsonAppState, 
        configure_middleware_addone_wrapped, 
        configure_middleware_addone_wrapped_fn, 
        configure_middleware_wrapped, 
        json_configs, 
        middleware_configure,
        cookie_configure
    }, 
    env_logger::{self, Env}, 
    openssl::ssl::{SslAcceptor, SslFiletype, SslMethod}
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
                // If you want to test anything here just comment other functions so you can try
                web::scope("")
                    .app_data(json_state.clone())
                    .guard(guard::Host("www.myapp.test"))
                    // .configure(middleware_configure)
                    // .configure(configure_middleware_wrapped)
                    // .configure(configure_middleware_addone_wrapped)
                    // .configure(configure_middleware_addone_wrapped_fn)
                    // .configure(cookie_configure)
                    .configure(|cfg | { 
                        json_configs(cfg , json_state.clone())
                    })
                    .configure(cookie_configure)
                    .wrap(custom_logger)
                    .wrap(
                        // create cookie based session middleware 
                        // can send data over the middleware to other functions 
                        SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0u8; 64]))
                        .cookie_secure(false)
                        .build()
                    )
            })
            .wrap(logger)
    })
    .bind_openssl("127.0.0.1:433", builder)?
    .run()
    .await
}
