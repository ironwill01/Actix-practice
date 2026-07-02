use {
    actix_cors::{
        Cors
    } 
};

pub(super) mod sign_cors {
    use super::*;

    pub fn cors_setting() -> Cors {
        Cors::default()
        .allowed_origin("https://www.myapp.test")
        .allowed_methods(vec!["GET" , "POST"])
        .max_age(3600)
    }
} 