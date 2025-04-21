use actix_web::{post, web, Result as AwResult};
use crate::api::PostForm;
use crate::AppState;
use maud::{ html, Markup };
use actix_multipart::form::MultipartForm;
use crate::get_utc;
use crate::RenderError;
use crate::queries::*;
use welds::{ WeldsError, prelude::* };


#[post("/post/{id}")]
pub async fn new_post(data: web::Data<AppState>, path : web::Path<(i64, )>, MultipartForm(form) : MultipartForm<PostForm>) -> AwResult<Markup> {
    let (board_id, ) = path.into_inner();
    if let Some(image) = form.image.file_name {
        if image == "" {
            return Err(RenderError::BadImage.into());
        }
        let image = format!("{}-{}", get_utc(), image);
        std::fs::copy(form.image.file.path(), format!("{}/{}", data.config.images, image)).map_err(|_| RenderError::FilesystemError)?;
        let mut post = Post::new_post(form.username.into_inner(), form.content.into_inner(), image, board_id);
        post.save(&data.welds).await.map_err(|e| <WeldsError as Into<Box<dyn std::error::Error>>>::into(e))?;
    }
    Ok(
        html! {
            meta http-equiv="refresh" content={"0; url=/b/" (board_id)};
        }
    )
}
