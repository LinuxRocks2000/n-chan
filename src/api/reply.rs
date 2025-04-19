use actix_web::{post, web, Result as AwResult};
use crate::api::PostForm;
use crate::AppState;
use maud::{ html, Markup };
use actix_multipart::form::MultipartForm;
use crate::get_utc;
use crate::RenderError;
use crate::queries::*;


#[post("/reply/{id}/{board}")]
async fn reply_to_post(data: web::Data<AppState>, path : web::Path<(i64, i64)>, MultipartForm(form) : MultipartForm<PostForm>) -> AwResult<Markup> {
    let (post_id, board_id) = path.into_inner();
    let image = if let Some(name) = form.image.file_name {
        if name != "" {
            let name = format!("{}-{}", get_utc(), name);
            form.image.file.persist(format!("{}/{}", data.config.images, name)).map_err(|_| RenderError::FilesystemError)?;
            Some(name)
        }
        else { None }
    } else { None };

    let post = Post::new_reply(form.username.into_inner(), form.content.into_inner(), image, Identifier::Post(post_id));
    post.send(&*data.db.lock().map_err(|_| RenderError::MutexingFailure)?)?;

    Ok(
        html! {
            meta http-equiv="refresh" content={"0; url=/b/" (board_id)};
        }
    )
}

