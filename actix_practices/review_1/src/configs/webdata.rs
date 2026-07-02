use {
    actix_web::web, askama::Template, derive_more::Display, serde::{
        Deserialize , Serialize
    }, std::{
        sync::Mutex
    },
};

#[derive(Debug , Clone , Display , Deserialize , Serialize)]
#[display("{} : {}" , username , message)]
pub struct UserMessage {
    username : String ,
    message : String ,
}

pub struct UserState {
    datas : Mutex<Vec<UserMessage>>
}

#[derive(Template)]
#[template(path = "homepage.html")] 
pub struct UserTemplate {
    messages : Vec<UserMessage>
}

impl UserMessage {
    // use when need to init default for the mutex part    
    pub fn new(u : String , u_message : String) -> Self {
        Self { username : u , message : u_message }
    }

    // use when data need to be pushed into the vec 
    // pub fn from(rsh : &UserMessage) -> Self {
    //     Self { 
    //         username : rsh.username.clone() , 
    //         message : rsh.message.clone()
    //     }
    // }
}

impl UserState {
    pub fn new() -> web::Data<Self> {
        web::Data::new(Self { datas : Mutex::new(Vec::new()) })
    }

    pub fn push(&self , data : UserMessage) -> () {
        match self.datas.lock() {
            Ok(mut vec) => {
                vec.push(data);
            } ,
            Err(err) => {
                eprintln!("Mutex guard is poisoned : {}" , err)
            }
        };
    }

    // pub fn get_vec(&self) -> Vec<UserMessage> {
    //     let vec = match self.datas.lock() {
    //         Ok(vec) => {
    //             vec
    //         } ,
    //         Err(err) => {
    //             eprintln!("Mutex guard is poisoned : {}" , err);
    //             panic!()
    //         }
    //     }.to_vec();

    //     vec.clone()
    // }
}

impl UserTemplate {
    pub fn new(data : &UserState) -> Self {
        let user_data = match data.datas.lock() {
            Ok(vec) => vec ,
            Err(err) => {
                eprintln!("Mutex guard poisoned : {}" , err);
                panic!()
            }
        };
        Self { messages : user_data.to_vec()}
    }
}