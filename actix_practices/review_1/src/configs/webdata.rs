use {
    actix_web::{
        web ,
    } ,
    serde::{
        Deserialize , Serialize
    } ,
    derive_more::{
        Display
    } ,
    tokio::{
        sync::Mutex
    } ,
};


#[derive(Debug , Display , Deserialize , Serialize)]
#[display("{} : {}" , username , message)]
pub struct UserMessage {
    username : String ,
    message : String ,
}


impl UserMessage {
    
    // use when need to init default for the mutex part    
    pub fn new() -> web::Data<Mutex<Vec<Self>>> {
        web::Data::new( Mutex::new(
                Vec:: new()
            )
        )
    }

    // use when data need to be pushed into the vec 
    pub fn from(u : String , u_message : String) -> Self {
        Self { 
            username : u , 
            message : u_message
        }
    }
}