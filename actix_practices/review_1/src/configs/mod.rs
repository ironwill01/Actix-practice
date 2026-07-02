pub mod default;
pub mod signname;
pub mod webdata;
pub mod cors;
// Maybe i add utils later

const TEMPLATE_PATH: &str = "./actix_practices/review_1/src/templates";

pub(super) use default::{
    default_configs
};

pub(super) use signname::{
    signpage_config
};

pub(super) use webdata::{
    UserMessage ,
    UserState ,
    UserTemplate
};

pub(super) use cors::{
    sign_cors::{
        cors_setting
    }
};