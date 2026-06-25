pub mod default;
pub mod signname;
pub mod webdata;
// Maybe i add utils later

const TEMPLATE_PATH: &str = "./actix_practices/review_1/src/templates";

pub use default::{
    default_configs
};

pub use signname::{
    signpage_config
};

pub(super) use webdata::{
    UserMessage ,
    UserState ,
    UserTemplate
};