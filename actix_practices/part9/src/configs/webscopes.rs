use {
    actix_web::{
        App, Error, HttpRequest, HttpResponse, Result,
        body::{self, MessageBody},
        get,
        http::{
            self, StatusCode,
            header::{ContentEncoding, ContentType},
        },
        rt::pin,
        test, web,
    },
    futures::{future, stream},
    std::{sync::Mutex, task::Poll},
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
        // Create an app instance to test our calls
        let app = test::init_service(App::new().service(get_index)).await;
        // Load the request as a var
        let req = test::TestRequest::default()
            // inseting plain text since data is just string and there is no format
            .insert_header(ContentType::plaintext())
            .to_request();
        // we get the response and we can test the reuslt
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    // Lets make test for AppStates
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

        // We can read data just after getting our response
        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        assert_eq!(body_str, "john victor gustavo");
    }

    // This part is about streaming testing which we can turn the data
    // into parts and then test it

    #[get("/server_event")]
    async fn server_event(_req: HttpRequest) -> HttpResponse {
        // Starting from 5 we can use other numbers
        let mut counter: usize = 5;

        let server_events =
            stream::poll_fn(move |_cx| -> Poll<Option<Result<web::Bytes, Error>>> {
                if counter == 0 {
                    return Poll::Ready(None);
                }
                let payload = format!("data: {}\n\n", counter);
                counter -= 1;
                Poll::Ready(Some(Ok(web::Bytes::from(payload))))
            });

        HttpResponse::build(StatusCode::OK)
            // Since we dont import any crate or lib we have to write the content type ourselves
            .insert_header((http::header::CONTENT_TYPE, "text/event-stream"))
            // Same as this endcoding
            .insert_header(ContentEncoding::Identity)
            .streaming(server_events)
    }

    #[actix_web::test]
    async fn test_stream_chunk() {
        let app =
            test::init_service(App::new().service(server_event))
                .await;
        let req = test::TestRequest::get().uri("/server_event").to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = resp.into_body();
        pin!(body);

        // now we check the data chunk by chunk

        // first chunk
        let bytes = future::poll_fn(|cx| body.as_mut().poll_next(cx)).await;
        assert_eq!(
            bytes.unwrap().unwrap(),
            web::Bytes::from_static(b"data: 5\n\n")
        );

        // second chunk
        let bytes = future::poll_fn(|cx| body.as_mut().poll_next(cx)).await;
        assert_eq!(
            bytes.unwrap().unwrap(),
            web::Bytes::from_static(b"data: 4\n\n")
        );

        // remaining part
        for i in 0..3 {
            let expected_data = format!("data: {}\n\n", 3 - i);
            let bytes = future::poll_fn(|cx| body.as_mut().poll_next(cx)).await;
            assert_eq!(bytes.unwrap().unwrap(), web::Bytes::from(expected_data));
        }
    }
}
