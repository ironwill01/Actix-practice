use {
    actix_web::{
        App, HttpResponse, Result, get,
        http::{StatusCode, header::ContentType},
        test, web,
    },
    std::sync::Mutex,
};

// this part is about testing in actix framework
#[cfg(test)]
pub(crate) mod cfg {

    use super::*;

    #[get("/")]
    async fn get_index() -> Result<HttpResponse> {
        Ok(HttpResponse::build(StatusCode::OK)
            .insert_header(ContentType::html())
            .body("hello world !"))
    }

    #[actix_web::test]
    async fn test_get() {
        let app = test::init_service(App::new().service(get_index)).await;
        let req = test::TestRequest::default()
            .insert_header(ContentType::plaintext())
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    // lets make test for AppStates
    struct User {
        user: String,
    }

    struct AppState {
        appdata: Mutex<Vec<User>>,
    }

    fn appstate() -> Result<web::Data<AppState>> {
        let user_vector = vec!["john", "victor", "gustavo"]
            .iter()
            .map(|u| User {
                user: u.to_string(),
            })
            .collect::<Vec<User>>();
        Ok(web::Data::new(AppState {
            appdata: Mutex::new(user_vector),
        }))
    }

    #[get("/test_appstate")]
    async fn get_app_state(data: web::Data<AppState>) -> Result<HttpResponse> {
        let _ = match data.appdata.lock() {
            Ok(users) => {
                return Ok(HttpResponse::build(StatusCode::OK)
                    .insert_header(ContentType::plaintext())
                    .body(
                        users
                            .iter()
                            .map(|u| format!("{}", u.user))
                            .collect::<Vec<_>>()
                            .join(" "),
                    ));
            }
            Err(err) => {
                eprintln!("Mutex guard poisoned : {}", err);
                return Ok(HttpResponse::InternalServerError().body("Internal server error!"));
            }
        };
    }

    #[actix_web::test]
    async fn test_appstate() {
        let data = appstate().expect("Could not create the sample data!");

        let app =
            test::init_service(App::new().app_data(data.clone()).service(get_app_state)).await;

        let req = test::TestRequest::get().uri("/test_appstate").to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        assert_eq!(body_str, "john victor gustavo");
    }
}
