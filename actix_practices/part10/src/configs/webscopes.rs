use {
    actix_web::{
        Error, HttpResponse, Result,
        body::{BoxBody, MessageBody, to_bytes},
        dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
        get,
        http::{StatusCode, header::ContentType},
        middleware::{Next, from_fn},
        web::{self, ServiceConfig},
    },
    futures_util::future::LocalBoxFuture,
    rand::random_range,
    std::future::{Ready, ready},
};

// Middleware one of the most important practices i should have
// middlewares are typically involved in following actions
// Pre-process the Request
// Post-process a Response
// Modify application state
// Access external services (redis, logging, sessions)

// Also middlewares are registered for each `App` , `scope` or `Resource`

pub mod scopes {

    use futures::FutureExt;

    use super::*;

    // There are two steps in middleware processing.
    // 1. Middleware initialization, middleware factory gets called with
    //    next service in chain as parameter.
    // 2. Middleware's call method gets called with normal request.
    pub(crate) struct PrintSomething;

    // Middleware factory is `Transform` trait
    // `S` - type of the next service
    // `B` - type of response's body
    impl<S, Body> Transform<S, ServiceRequest> for PrintSomething
    where
        S: Service<ServiceRequest, Response = ServiceResponse<Body>, Error = Error>,
        S::Future: 'static,
        Body: 'static,
    {
        // Local type binding to the trait object to create our own middleware
        type Response = ServiceResponse<Body>;
        type Error = Error;
        type InitError = ();
        type Transform = PrintSomethingMiddleware<S>;
        type Future = Ready<Result<Self::Transform, Self::InitError>>;

        fn new_transform(&self, service: S) -> Self::Future {
            ready(Ok(PrintSomethingMiddleware { service }))
        }
    }

    pub(crate) struct PrintSomethingMiddleware<S> {
        service: S,
    }

    impl<S, Body> Service<ServiceRequest> for PrintSomethingMiddleware<S>
    where
        S: Service<ServiceRequest, Response = ServiceResponse<Body>, Error = Error>,
        S::Future: 'static,
        Body: 'static,
    {
        type Response = ServiceResponse<Body>;
        type Error = Error;
        type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

        forward_ready!(service);

        fn call(&self, req: ServiceRequest) -> Self::Future {
            println!("Printing from service your request was: {}", req.path());

            let fut = self.service.call(req);

            Box::pin(async move {
                let res = fut.await?;
                println!("Response!");
                Ok(res)
            })
        }
    }

    // now let create an function and config so we can use it
    pub fn middleware_configure(cfg: &mut ServiceConfig) {
        cfg.service({
            web::scope("/users")
                .service(random)
                .wrap(PrintSomething)
                .service(random_one)
                .wrap(AddOne)
        });
    }

    #[get("/num")]
    async fn random() -> Result<HttpResponse> {
        Ok(HttpResponse::build(StatusCode::OK)
            .insert_header(ContentType::html())
            .body(format!(
                "random number of today : {}",
                random_range(1..=100)
            )))
    }

    // Now lets create a middleware which add number to this page
    pub(crate) struct AddOne;
    // Explain to Transform trait about the Middleware factory
    // S for service and B for body
    impl<S, B> Transform<S, ServiceRequest> for AddOne
    where
        // Explain which type of service with which outcome do we need
        // In our case we need BoxBody since we want to change the data of the page
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
        S::Future: 'static,
        B: MessageBody + 'static,
    {
        // Type of stuff we need in factory
        type Response = ServiceResponse<BoxBody>;
        type Error = Error;
        type InitError = ();
        type Transform = AddOneService<S>;
        type Future = Ready<Result<Self::Transform, Self::InitError>>;

        // Transform begins
        fn new_transform(&self, service: S) -> Self::Future {
            ready(Ok(AddOneService { service }))
        }
    }

    // Service struct
    pub(crate) struct AddOneService<S> {
        service: S,
    }

    // This is where service is actually created and we can process data before or post call
    impl<S, B> Service<ServiceRequest> for AddOneService<S>
    where
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
        S::Future: 'static,
        B: MessageBody + 'static,
    {
        type Response = ServiceResponse<BoxBody>;
        type Error = Error;
        type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

        forward_ready!(service);

        fn call(&self, req: ServiceRequest) -> Self::Future {
            println!("Printing from service your request was : {}", req.path());

            let fut = self.service.call(req);

            // Now we just want to add one number to our random number
            Box::pin(async move {
                match fut.await {
                    Ok(service) => {
                        // we get the body and requset
                        let (req, res) = service.into_parts();
                        let body = res.into_body(); // HttpResponse<B> → B
                        let bytes = to_bytes(body).await;

                        match bytes {
                            Ok(data) => {
                                let original_body = String::from_utf8_lossy(&data);

                                let modified_text = if let Some(num) = original_body
                                    .split(':')
                                    .last()
                                    .and_then(|s| s.trim().parse::<i32>().ok())
                                {
                                    println!("num is : {} replace with new num {}", &num, &num + 1);
                                    original_body.replace(&num.to_string(), &(num + 1).to_string())
                                } else {
                                    original_body.into_owned()
                                };

                                Ok(ServiceResponse::new(
                                    req,
                                    HttpResponse::build(StatusCode::OK)
                                        .insert_header(ContentType::html())
                                        .body(modified_text),
                                ))
                            }
                            Err(_err) => Ok(ServiceResponse::new(
                                req,
                                HttpResponse::InternalServerError().body("Error from middleware"),
                            )),
                        }
                    }
                    Err(err) => Err(err),
                }
            })
        }
    }

    #[get("/numone")]
    async fn random_one() -> Result<HttpResponse> {
        Ok(HttpResponse::build(StatusCode::OK)
            .insert_header(ContentType::html())
            .body(format!(
                "random number of today : {}",
                random_range(1..=100)
            )))
    }

    // we can also wrap a middleware in warp_fn() in alternative
    // ill add it in next commit i assume
    pub fn configure_middleware_wrapped(cfg: &mut ServiceConfig) {
        cfg.service(
            web::scope("")
                .wrap_fn(|req, srv| {
                    println!("Request called: {}", req.path());
                    srv.call(req).map(|res| {
                        println!("Calling response ...");
                        res
                    })
                })
                .service(random_one_wrapped),
        );
    }

    #[get("/numone_wrapped")]
    async fn random_one_wrapped() -> Result<HttpResponse> {
        Ok(HttpResponse::build(StatusCode::OK)
            .insert_header(ContentType::html())
            .body(format!(
                "random number of today : {}",
                random_range(1..=100)
            )))
    }

    // as you can see its almos the same logic as the last one but with less detail
    // if you whave something complex you better use factory no wrap_fn()
    pub fn configure_middleware_addone_wrapped(cfg: &mut ServiceConfig) {
        cfg.service(
            web::scope("")
                .wrap_fn(|req, service| {
                    let fut = service.call(req);
                    Box::pin(async move {
                        match fut.await {
                            Ok(response) => {
                                let (httpreq, body) = response.into_parts();
                                let bytes = to_bytes(body.into_body()).await?;

                                let string_body = String::from_utf8_lossy(&bytes);

                                // and holy bad syntax for this one ngl this pattern is cancer let me talk out of
                                // tutorial for myself
                                let modified_body = if let Some(num) = string_body
                                    .split(":")
                                    .last()
                                    .and_then(|number| number.trim().parse::<i32>().ok())
                                {
                                    println!(
                                        "found number {} replacing with number {}",
                                        &num,
                                        (&num + 1)
                                    );
                                    string_body.replace(&num.to_string(), &(&num + 1).to_string())
                                } else {
                                    string_body.to_string()
                                };

                                Ok(ServiceResponse::new(
                                    httpreq,
                                    HttpResponse::build(StatusCode::OK)
                                        .insert_header(ContentType::html())
                                        .body(modified_body),
                                ))
                            }
                            Err(err) => Err(err),
                        }
                    })
                })
                .service(random_one_addone_warapped),
        );
    }

    // lets create addone to the function using warp_fn()
    #[get("/numone_wrapped_addone")]
    async fn random_one_addone_warapped() -> Result<HttpResponse> {
        Ok(HttpResponse::build(StatusCode::OK)
            .insert_header(ContentType::html())
            .body(format!(
                "random number of today : {}",
                random_range(1..=100)
            )))
    }

    // we can also have functions as middleware with from_fn
    // lets create addone to the function using warp_fn()
    pub fn configure_middleware_addone_wrapped_fn(cfg: &mut ServiceConfig) {
        cfg.service(
            web::scope("")
                .wrap(from_fn(middle_ware_fn))
                .service(random_one_addone_warapped_fn),
        );
    }

    async fn middle_ware_fn(
        req: ServiceRequest,
        next: Next<impl MessageBody + 'static>,
    ) -> Result<ServiceResponse<impl MessageBody>, Error> {
        // now you can pre process anything here
        println!("Loading addone middleware from fn ...");
        // then call next
        match next.call(req).await {
            Ok(response) => {
                let (req, body) = response.into_parts();
                let bytes = to_bytes(body.into_body().boxed()).await?;

                let string_body = String::from_utf8_lossy(&bytes);

                let modified_body = if let Some(num) = string_body
                    .split(":")
                    .last()
                    .and_then(|number| number.trim().parse::<i32>().ok())
                {
                    println!("found number {} replacing with number {}", &num, (&num + 1));
                    string_body.replace(&num.to_string(), &(&num + 1).to_string())
                } else {
                    string_body.to_string()
                };

                return Ok(ServiceResponse::new(
                    req,
                    HttpResponse::build(StatusCode::OK)
                        .insert_header(ContentType::html())
                        .body(modified_body),
                ));
            }
            Err(err) => {
                return Err(err);
            }
        };
        //in our case we just go with addone
    }

    #[get("/numone_wrapped_addone_fn")]
    async fn random_one_addone_warapped_fn() -> Result<HttpResponse> {
        Ok(HttpResponse::build(StatusCode::OK)
            .insert_header(ContentType::html())
            .body(format!(
                "random number of today : {}",
                random_range(1..=100)
            )))
    }
}
