// todo: do error handling properly (currently it's shockingly bad)

use actix_web::{get, post, web, App, HttpServer, Result as AwResult};
use actix_multipart::Multipart;
use maud::{html, Markup, DOCTYPE, PreEscaped};
use std::io;
use rusqlite::{Connection, Result};
use std::sync::{Arc, Mutex};
use futures_util::TryStreamExt;


fn get_utc() -> i64 {
    chrono::Utc::now().timestamp()
}


struct Board {
    name : String,
    desc : String,
    id : i64
}


#[derive(Debug)]
struct Post {
    username : String,
    content : String,
    image : String,
    id : i64,
    board : i64,
    time : i64,
}


fn pageroot(db : &Connection, content : impl maud::Render) -> PreEscaped<String> {
    let mut stmt = db.prepare("SELECT * FROM boards").unwrap();
    let boards = stmt.query_map([], |row| {
        Ok(Board{
            name : row.get(0)?,
            desc : row.get(1)?,
            id : row.get(2)?
        })
    }).unwrap();
    html! {
        (DOCTYPE)
        html {
            head {
                title { "N-CHAN: THE BEST IMAGEBOARD" }
                link rel="stylesheet" href="/res/main.css";
                link rel="icon" href="/res/yotsubm.png";
            }
            body {
                div id="top" {
                    a href="/" { img src="/res/widebanner.png"; }
                    p {
                        @for o_b in boards {
                            {
                                @if let Ok(o_b) = o_b {
                                    a href={ "/b/" (o_b.id) } { "/" (o_b.name) "/" }
                                }
                            }
                        }
                    }
                }
                (content)
                script src="/res/main.js" {}
            }
        }
    }
}


fn gen_post(db : &Connection, post : Post) -> Markup {
    let mut stmt = db.prepare("SELECT * FROM replies WHERE post=? ORDER BY time DESC LIMIT 10").unwrap();
    let replies = stmt.query_map([post.id], |row| {
        Ok(Post {
            username : row.get(0)?,
            content : row.get(1)?,
            image : row.get(3)?,
            board : -1,
            id : -1,
            time : row.get(4)?
        })
    }).unwrap();
    html!{
        div class="post" {
            a href={"/img/" (post.image)} { img src={ "/img/" (post.image) }; }
            i { "user:" (post.username) }
            br;
            (post.content)
            br; br;
            form action = { "/reply/" (post.id) "/" (post.board)} method="post" enctype="multipart/form-data" onsubmit="before_submit()" class="replyform" {
                "username: " input type="text" name="username" class="username"; br;
                textarea name="content" {} br;
                "image (optional): " input type="file" name="image";
                input type="submit" value="Post Reply";
            }
            @for reply in replies {
                @if let Ok(reply) = reply {
                    br;
                    div class="reply" {
                        @if reply.image != "" {
                            a href={"/img/" (reply.image)} { img src={ "/img/" (reply.image) }; }
                        }
                        i { "user:" (reply.username) " replies" }
                        br;
                        (reply.content)
                    }
                }
            }
        }
    }
}


#[get("/")]
async fn index(data: web::Data<Arc<Mutex<AppState>>>) -> AwResult<Markup> {
    Ok(pageroot(&data.lock().unwrap().db, html! {
        p {
            "Q: this page fucking sucks! I wanna do something interesting! like browse pinterest because im a stupid libturd!"
        }
        p {
            "A: click a fucking board you stupid bitch"
        }
    }))
}

#[get("/b/{id}")]
async fn board(data: web::Data<Arc<Mutex<AppState>>>, path : web::Path<(i64, )>) -> AwResult<Markup> {
    let (board_id, ) = path.into_inner();
    let board = {
        let db = &data.lock().unwrap().db;
        let mut board_name = db.prepare("SELECT * FROM boards WHERE id=?1").unwrap();
        let mut rows = board_name.query([board_id]).unwrap();
        let next = rows.next().unwrap().unwrap();
        Board {
            name : next.get(0).unwrap(),
            desc : next.get(1).unwrap(),
            id : next.get(2).unwrap()
        }
    };
    let page = {
        let db = &data.lock().unwrap().db;
        let mut stmt = db.prepare("SELECT * FROM posts WHERE board=?1 ORDER BY time DESC LIMIT 50").unwrap();
        let mut posts = stmt.query_map([board_id], |row| {
            Ok(Post {
                username : row.get(0)?,
                content : row.get(1)?,
                board : row.get(2)?,
                image : row.get(3)?,
                id : row.get(4)?,
                time : row.get(5)?
            })
        }).unwrap();
        html!{
            h1 { (board.name) }
            p id="desc" {
                i {
                    (board.desc)
                }
            }
            form action = { "/post/" (board_id)} method="post" enctype="multipart/form-data" onsubmit="before_submit()" id="postform" {
                "username: " input type="text" name="username" class="username"; br;
                textarea name="content" {} br;
                "image (required): " input type="file" name="image"; br;
                input type="submit" value="Post";
            }
            @for post in posts {
                @if let Ok(post) = post {
                    (gen_post(db, post))
                }
                @else {
                    "error loading post: " (format!("{:?}", post))
                }
            }
        }
    };
    Ok(
        pageroot(&data.lock().unwrap().db, page)
    )
}

use actix_multipart::form::{self, MultipartForm};

#[derive(Debug, MultipartForm)]
struct PostForm {
    username : form::text::Text<String>,
    content : form::text::Text<String>,
    #[multipart(limit = "20MB")]
    image : form::tempfile::TempFile
}


#[post("/post/{id}")]
async fn new_post(data: web::Data<Arc<Mutex<AppState>>>, path : web::Path<(i64, )>, MultipartForm(form) : MultipartForm<PostForm>) -> AwResult<Markup> {
    let (board_id, ) = path.into_inner();
    if let Some(name) = form.image.file_name {
        let name = format!("{}-{}", get_utc(), name);
        form.image.file.persist(format!("images/{}", name)).unwrap();
        let db = &data.lock().unwrap().db;
        let mut stmt = db.prepare("INSERT INTO posts(username, content, board, image, time) VALUES(?1, ?2, ?3, ?4, ?5)").unwrap();
        stmt.execute([form.username.into_inner(), form.content.into_inner(), board_id.to_string(), name, get_utc().to_string()]).unwrap();
    }
    Ok(
        html! {
            meta http-equiv="refresh" content={"0; url=/b/" (board_id)};
        }
    )
}

#[post("/reply/{id}/{board}")]
async fn reply_to_post(data: web::Data<Arc<Mutex<AppState>>>, path : web::Path<(i64, i64)>, MultipartForm(form) : MultipartForm<PostForm>) -> AwResult<Markup> {
    let (post_id, board_id) = path.into_inner();
    let image = if let Some(name) = form.image.file_name {
        if name != "" {
            let name = format!("{}-{}", get_utc(), name);
            form.image.file.persist(format!("images/{}", name)).unwrap();
            name
        }
        else { String::new() }
    } else { String::new() };

    let db = &data.lock().unwrap().db;
    let mut stmt = db.prepare("INSERT INTO replies(username, content, post, image, time) VALUES(?1, ?2, ?3, ?4, ?5)").unwrap();
    stmt.execute([form.username.into_inner(), form.content.into_inner(), post_id.to_string(), image, get_utc().to_string()]).unwrap();

    Ok(
        html! {
            meta http-equiv="refresh" content={"0; url=/b/" (board_id)};
        }
    )
}


struct AppState {
    db : Connection
}


#[actix_web::main]
async fn main() -> io::Result<()> {
    HttpServer::new(|| App::new()
            .app_data(web::Data::new(Arc::new(Mutex::new(AppState {
                db: Connection::open("n-chan.db").unwrap()
            }))))
            .service(index)
            .service(board)
            .service(actix_files::Files::new("/res", "static"))
            .service(actix_files::Files::new("/img", "images"))
            .service(new_post)
            .service(reply_to_post)
        )
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}

