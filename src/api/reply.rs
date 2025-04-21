use actix_web::{post, web, Result as AwResult};
use crate::api::PostForm;
use crate::AppState;
use maud::{ html, Markup };
use actix_multipart::form::MultipartForm;
use crate::get_utc;
use crate::RenderError;
use crate::queries::*;
use welds::WeldsError;


#[post("/reply/{id}/{target}/{tid}")]
async fn reply_to_post(data: web::Data<AppState>, path : web::Path<(i64, String, i64)>, MultipartForm(form) : MultipartForm<PostForm>) -> AwResult<Markup> {
    let (post_id, target, tid) = path.into_inner();
    let image = if let Some(name) = form.image.file_name {
        if name != "" {
            let name = format!("{}-{}", get_utc(), name);
            std::fs::copy(form.image.file.path(), format!("{}/{}", data.config.images, name)).map_err(|_| RenderError::FilesystemError)?;
            Some(name)
        }
        else { None }
    } else { None };

    let mut reply = Reply::new_reply(form.username.into_inner(), form.content.into_inner(), image, post_id);
    reply.save(&data.welds).await.map_err(|e| <WeldsError as Into<Box<dyn std::error::Error>>>::into(e))?;

    Ok(
        html! {
            @if target == "board" {
                meta http-equiv="refresh" content={"0; url=/b/" (tid)};
            }
            @else {
                meta http-equiv="refresh" content={"0; url=/post/" (tid)};
            }
        }
    )
}

