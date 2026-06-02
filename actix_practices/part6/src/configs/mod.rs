pub mod webscopes;

// gotta put what ever func i want in this pub use part ( clean code reasons ) ( i dont write clean anyway )
pub use {
    webscopes::scopes::{simple_index , DataBase , UserAppState , UserDataBase}
};