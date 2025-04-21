// todo: do error handling properly (currently it's shockingly bad)

pub const BOARD_PAGE_SIZE : i64 = 20;

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
use rand::prelude::*;


pub fn get_utc() -> i64 {
    chrono::Utc::now().timestamp()
}


#[derive(Debug)]
pub enum RenderError {
    QueryFailure(QueryError),
    MutexingFailure,
    FilesystemError,
    BadImage
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
    let replies = post.get_replies(db, 3).map_err(RenderError::QueryFailure)?;
    let num = post.id.unwrap();
    Ok(fragments::post(post, html!{
        @for reply in replies {
            br;
            (fragments::reply(reply))
        }
        i { a href={"/post/" (num) } {"see all replies"} }
    }, Identifier::Board(-1)))
}


#[get("/post/{id}")]
async fn view_post(data : web::Data<AppState>, path : web::Path<(i64,)>) -> AwResult<Markup> {
    let (post_id,) = path.into_inner();
    let post = Post::get_one_post(&*data.get_db()?, Identifier::Post(post_id))?;
    let replies = post.get_all_replies(&*data.get_db()?)?;
    Ok(fragments::pageroot(&*data.get_db()?, &data.config, html! {
        h1 {"/post/" (post_id) "/"}
        p style="text-align: center" { a href={"/b/" (post.target.unwrap())} {"back"} }
        (fragments::post(post, html! {
            @for reply in replies {
                (fragments::reply(reply))
            }
        }, Identifier::Post(post_id)))
    })?)
}


#[get("/")]
async fn index(data: web::Data<AppState>) -> AwResult<Markup> {
    Ok(fragments::pageroot(&*data.get_db()?, &data.config, html! {
        h1 { "Welcome to " (data.config.title) "!" }
        p { "This is an instance of " a href="https://github.com/LinuxRocks2000/n-chan" {"n-chan"} ". Check out some boards!"}
        p {
            a href="/rand/" { "random board" }
        }
        p {
            a href="https://landgreen.github.io/n-gon" { "play n-gon" }
        }
        p {
             b { "rules" } br;
             "don't post in /yuri/. don't impersonate weirdpusheen. don't malign the holy word of yotsuba."
        }
    })?)
}

async fn board(data: web::Data<AppState>, board_id : i64, page : i64) -> AwResult<Markup> {
    let board = Board::get_one_board(&*data.get_db()?, Identifier::Board(board_id))?;
    let out = {
        let db = &*data.get_db()?;
        let posts = Post::get_from_board(db, queries::Identifier::Board(board_id), page)?;
        html!{
            h1 { "/" (board.name) "/" @if page != 0 { (page) "/" } }
            p id="desc" {
                i {
                    (board.topic)
                }
            }
            p style="text-align: center;" {
                @if page > 0 {
                    a href={ "/b/" (board_id) "/" (page - 1)} {"previous"}
                }
                " page " (page + 1) " "
                @if posts.len() == BOARD_PAGE_SIZE as usize {
                    a href={ "/b/" (board_id) "/" (page + 1)} {"next"}
                }
            }
            @if page == 0 {
                (fragments::post_box(&format!("/post/{}", board_id), "postform"))
            }
            @else {
                p style="text-align: center;" { a href={ "/b/" (board_id) } {"back to /" (board.name) "/ front page"} }
            }
            @if posts.len() > 0 {
                @for post in posts {
                    ({ gen_post(db, post)? })
                }
            }
            @else {
                p style="text-align: center" { i { "no posts here. try " a href="/rand/" {"/rand/"} "?" } }
            }
        }
    };
    Ok(
        fragments::pageroot(&*data.get_db()?, &data.config, out)?
    )
}

#[get("/b/{id}/{page}")]
async fn board_paged(data: web::Data<AppState>, path : web::Path<(i64, i64 )>) -> AwResult<Markup> {
    let (board_id, page) = path.into_inner();
    board(data, board_id, page).await
}

#[get("/b/{id}")]
async fn board_firstpage(data: web::Data<AppState>, path : web::Path<(i64, )>) -> AwResult<Markup> {
    let (board_id, ) = path.into_inner();
    board(data, board_id, 0).await
}

#[get("/{name}/")]
async fn board_by_name(data: web::Data<AppState>, path : web::Path<(String, )>) -> AwResult<Markup> {
    let (name, ) = path.into_inner();
    let id = if name == "rand" {
        let mut rng = rand::thread_rng();
        rng.gen_range(0..Board::count_boards(&*data.get_db()?)?)
    }
    else {
        let brd = Board::get_by_name(&*data.get_db()?, name)?;
        brd.id.unwrap() as usize
    };
    Ok(
        html! {
            meta http-equiv="refresh" content={"0; url=/b/" (id)};
        }
    )
}

#[derive(Clone)]
struct AppState {
    db : Arc<Mutex<Connection>>,
    config : Config
}


impl AppState {
    fn get_db<'a>(&'a self) -> Result<std::sync::MutexGuard<'a, Connection>, RenderError> {
        Ok(self.db.lock().map_err(|_| RenderError::MutexingFailure)?)
    }
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
            .service(board_firstpage)
            .service(board_paged)
            .service(board_by_name)
            .service(view_post)
            .service(actix_files::Files::new("/res", "static"))
            .service(actix_files::Files::new("/img", &image_dir))
            .service(api::new_post)
            .service(api::reply_to_post)
        )
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}

