mod configs;

use {
    configs::{
        default_configs
    },
    actix_files::Files
    , actix_web::{
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

    unsafe {
        std::env::set_var("RUST_LOG", "info");
        std::env::set_var("RUST_BACKTRACE", "1");
    }

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

    println!("Starting HTTPS server at https://www.myapp.test:443");

    HttpServer::new(move || {
        let logger = Logger::new("%s %a %{User-Agent}i");
        
        App::new().service({
            web::scope("")
            .wrap(logger)

            // Now lets load static_files into the server
            // Personal note : im proud thats it

            // To serve files from specific directories and sub-directories, Files can be used. 
            // Files must be registered with an App::service() method, otherwise it will be unable to serve sub-paths.

            // By default files listing for sub-directories is disabled. Attempt to load directory listing will return 404 Not Found response. 
            // To enable files listing, use Files::show_files_listing() method.

            // Instead of showing files listing for a directory, it is possible to redirect to a specific index file. 
            // Use the Files::index_file() method to configure this redirect.
             
            .service(Files::new("/static" , "./actix_practices/part12/src/templates").show_files_listing())
            .guard(guard::Host("www.myapp.test"))
            .configure(default_configs)
        })
    })
    .bind_openssl("127.0.0.1:443", builder)?
    .run()
    .await

}   