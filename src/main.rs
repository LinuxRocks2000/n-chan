// todo: do error handling properly (currently it's shockingly bad)

pub const BOARD_PAGE_SIZE : i64 = 20;

use actix_web::{get, web, App, HttpServer, Result as AwResult};
use maud::{html, Markup};
use std::io;
use std::sync::{Arc, Mutex};
use welds::connections::any::AnyClient as WeldsClient;

mod migrations;

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
    MutexingFailure,
    FilesystemError,
    BadImage
}


impl actix_web::error::ResponseError for RenderError {}


impl std::fmt::Display for RenderError {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for RenderError {}


async fn gen_post(db : &WeldsClient, post : Post, board : i64) -> Result<Markup, Box<dyn std::error::Error>> {
    let replies = post.get_replies(db, Some((3, 0))).await?;
    let num = post.id;
    Ok(fragments::post(post, html!{
        @for reply in replies {
            br;
            (fragments::reply(reply))
        }
        i { a href={"/post/" (num) } {"see all replies"} }
    }, board, false))
}


#[get("/post/{id}")]
async fn view_post(data : web::Data<AppState>, path : web::Path<(i64,)>) -> AwResult<Markup> {
    let (post_id,) = path.into_inner();
    let post = Post::get_one_post(&data.welds, post_id).await?;
    let replies = post.get_replies(&data.welds, None).await?;
    Ok(fragments::pageroot(&data.welds, &data.config, html! {
        h1 {"/post/" (post_id) "/"}
        p style="text-align: center" { a href={"/b/" (post.board)} {"back"} }
        (fragments::post(post, html! {
            @for reply in replies {
                (fragments::reply(reply))
            }
        }, post_id, true))
    }).await?)
}


#[get("/")]
async fn index(data: web::Data<AppState>) -> AwResult<Markup> {
    Ok(fragments::pageroot(&data.welds, &data.config, html! {
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
    }).await?)
}

async fn board(data: web::Data<AppState>, board_id : i64, page : i64) -> AwResult<Markup> {
    let board = Board::get_one_board(&data.welds, board_id).await?;
    let out = {
        let posts = Post::get_from_board(&data.welds, board_id, page).await?;
        html!{
            h1 { "/" (board.name) "/" @if page != 0 { (page) "/" } }
            p id="desc" {
                i {
                    (board.desc)
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
                    ({ gen_post(&data.welds, post, board_id).await? })
                }
            }
            @else {
                p style="text-align: center" { i { "no posts here. try " a href="/rand/" {"/rand/"} "?" } }
            }
        }
    };
    Ok(
        fragments::pageroot(&data.welds, &data.config, out).await?
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
        rng.gen_range(1..Board::count_boards(&data.welds).await?)
    }
    else {
        let brd = Board::get_by_name(&data.welds, name).await?;
        brd.id
    };
    Ok(
        html! {
            meta http-equiv="refresh" content={"0; url=/b/" (id)};
        }
    )
}

#[derive(Clone)]
struct AppState {
    config : Config,
    welds : WeldsClient
}


#[actix_web::main]
async fn main() -> io::Result<()> {
    let config = get_config();
    let image_dir = config.images.clone();

    let welds = welds::connections::connect(&config.database).await.unwrap();
    migrations::up(&welds).await.unwrap();

    let state = AppState {
        config,
        welds
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
