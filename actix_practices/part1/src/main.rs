pub mod configs;
use actix_web::{App, HttpServer, web};

use configs::{dynamic_scope, init_config, static_scope};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().configure(init_config).service(
            web::scope("/scopes")
                .configure(static_scope)
                .configure(dynamic_scope),
        )
    })
    .bind(("127.0.0.1", 80))?
    .run()
    .await
}
