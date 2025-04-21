use actix_web::{post, web, Result as AwResult};
use crate::api::PostForm;
use crate::AppState;
use maud::{ html, Markup };
use actix_multipart::form::MultipartForm;
use crate::get_utc;
use crate::RenderError;
use crate::queries::*;


#[post("/post/{id}")]
pub async fn new_post(data: web::Data<AppState>, path : web::Path<(i64, )>, MultipartForm(form) : MultipartForm<PostForm>) -> AwResult<Markup> {
    let (board_id, ) = path.into_inner();
    if let Some(image) = form.image.file_name {
        if image == "" {
            return Err(RenderError::BadImage.into());
        }
        let image = format!("{}-{}", get_utc(), image);
        std::fs::copy(form.image.file.path(), format!("{}/{}", data.config.images, image)).map_err(|_| RenderError::FilesystemError)?;
        let post = Post::new_post(form.username.into_inner(), form.content.into_inner(), Some(image), Identifier::Board(board_id));
        post.send(&*data.db.lock().map_err(|_| RenderError::MutexingFailure)?)?;
    }
    Ok(
        html! {
            meta http-equiv="refresh" content={"0; url=/b/" (board_id)};
        }
    )
}
