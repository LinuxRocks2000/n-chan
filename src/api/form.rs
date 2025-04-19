use actix_multipart::form::{self, MultipartForm};

#[derive(Debug, MultipartForm)]
pub struct PostForm {
    pub username : form::text::Text<String>,
    pub content : form::text::Text<String>,
    #[multipart(limit = "20MB")]
    pub image : form::tempfile::TempFile
}
