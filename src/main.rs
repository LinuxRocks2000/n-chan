// todo: do error handling properly (currently it's shockingly bad)

use actix_web::{get, web, App, HttpServer, Result as AwResult};
use maud::{html, Markup};
use std::io;
use rusqlite::{Connection, Result};
use std::sync::{Arc, Mutex};

mod queries;
use queries::*;

mod config;
use config::*;

mod fragments;

mod api;


pub fn get_utc() -> i64 {
    chrono::Utc::now().timestamp()
}


#[derive(Debug)]
pub enum RenderError {
    QueryFailure(QueryError),
    MutexingFailure,
    FilesystemError
}


impl actix_web::error::ResponseError for RenderError {}
impl actix_web::error::ResponseError for queries::QueryError {}


impl std::fmt::Display for RenderError {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for RenderError {}


fn gen_post(db : &Connection, post : Post) -> Result<Markup, RenderError> {
    let replies = post.get_replies(db, 0).map_err(RenderError::QueryFailure)?;
    Ok(html!{
        div class="post" {
            @if let Some(image) = post.image {
                a href={"/img/" (image)} { img src={ "/img/" (image) }; }
            }
            i { "user:" (post.username) }
            br;
            (post.content)
            br; br;
            (fragments::post_box(&format!("/reply/{}/{}", post.id.unwrap(), post.target.unwrap()), "replyform"))
            @for reply in replies {
                br;
                div class="reply" {
                    @if let Some(image) = reply.image {
                        a href={"/img/" (image)} { img src={ "/img/" (image) }; }
                    }
                    i { "user:" (reply.username) " replies" }
                    br;
                    (reply.content)
                }
            }
        }
    })
}


#[get("/")]
async fn index(data: web::Data<AppState>) -> AwResult<Markup> {
    Ok(fragments::pageroot(&*data.db.lock().map_err(|_| RenderError::MutexingFailure)?, &data.config, html! {
        p {
            "Q: this page fucking sucks! I wanna do something interesting! like browse pinterest because im a stupid libturd!"
        }
        p {
            "A: click a fucking board you stupid bitch"
        }
    })?)
}

#[get("/b/{id}")]
async fn board(data: web::Data<AppState>, path : web::Path<(i64, )>) -> AwResult<Markup> {
    let (board_id, ) = path.into_inner();
    let board = Board::get_one_board(&*data.db.lock().map_err(|_| RenderError::MutexingFailure)?, Identifier::Board(board_id))?;
    let page = {
        let db = &data.db.lock().map_err(|_| RenderError::MutexingFailure)?;
        let posts = Post::get_from_board(db, queries::Identifier::Board(board_id), 0)?;
        html!{
            h1 { (board.name) }
            p id="desc" {
                i {
                    (board.topic)
                }
            }
            (fragments::post_box(&format!("/post/{}", board_id), "postform"))
            @for post in posts {
                (gen_post(db, post)?)
            }
        }
    };
    Ok(
        fragments::pageroot(&*data.db.lock().map_err(|_| RenderError::MutexingFailure)?, &data.config, page)?
    )
}

#[derive(Clone)]
struct AppState {
    db : Arc<Mutex<Connection>>,
    config : Config
}


#[actix_web::main]
async fn main() -> io::Result<()> {
    let config = get_config();
    let image_dir = config.images.clone();
    let db = get_database(&config);
    let state = AppState {
        db : Arc::new(Mutex::new(db)),
        config
    };
    HttpServer::new(move || App::new()
            .app_data(web::Data::new(state.clone()))
            .service(index)
            .service(board)
            .service(actix_files::Files::new("/res", "static"))
            .service(actix_files::Files::new("/img", &image_dir))
            .service(api::new_post)
            .service(api::reply_to_post)
        )
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}

