// sorta-ORM thingy
// contains structures that produce iterators over themselves queried from an sql database
// and also can commit to an sql database
// *also* contains setup routines
use crate::get_utc;
use crate::Config;
use rusqlite::{Connection, Result};


#[derive(Debug, Copy, Clone)]
pub enum Identifier {
    Board(i64),
    Post(i64),
    Reply(i64)
}


impl Identifier {
    pub fn unwrap(self) -> i64 {
        match self {
            Self::Board(i) => i,
            Self::Post(i) => i,
            Self::Reply(i) => i
        }
    }
}


#[derive(Debug)]
pub enum QueryError {
    LookupError(Identifier), // a thing could not be found
    BadIdent(Identifier), // an invalid identifier was specified
    SqlError(rusqlite::Error),
    NoResults // we got nothing when we should have gotten something
}


impl std::fmt::Display for QueryError {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for QueryError {}


#[derive(Debug)]
pub struct Post { // a post OR a reply
    pub username : String,
    pub content : String,
    pub image : Option<String>, // a filename; not every post has one
    pub id : Identifier,
    pub target : Identifier,
    pub time : i64
}


pub struct Board {
    pub name : String, // a few letters/numbers with no spaces. pretty printed in forward slashes. ex; /pol/
    pub topic : String,
    pub id : Identifier
}

impl Post {
    fn post_map(tp : impl Fn(i64) -> Identifier) -> impl Fn(&rusqlite::Row<'_>) -> rusqlite::Result<Post> {
        move |row: &rusqlite::Row<'_>| {
            let image_raw = row.get(3)?;
            let id = tp(row.get(4)?);
            Ok(Post {
                username : row.get(0)?,
                content : row.get(1)?,
                target : match id {
                    Identifier::Post(_) => Identifier::Board,
                    Identifier::Reply(_) => Identifier::Post,
                    Identifier::Board(_) => Identifier::Board
                }(row.get(2)?),
                image : if image_raw != "" { Some(image_raw) } else { None },
                id,
                time : row.get(5)?
            })
        }
    }

    pub fn get_one_post(conn : &rusqlite::Connection, id : Identifier) -> Result<Post, QueryError> {
        match id {
            Identifier::Post(i) => {
                let mut stmt = conn.prepare("SELECT * FROM posts WHERE id=?1 LIMIT 1").map_err(QueryError::SqlError)?;
                let mut rows = stmt.query_map([i.to_string()], Post::post_map(Identifier::Post)).map_err(QueryError::SqlError)?;
                if let Some(row) = rows.next() {
                    row.map_err(QueryError::SqlError)
                }
                else {
                    Err(QueryError::NoResults)
                }
            },
            _ => Err(QueryError::BadIdent(id))
        }
    }

    pub fn get_from_board(conn : &rusqlite::Connection, id : Identifier, page : i64) -> Result<Vec<Post>, QueryError> {
        match id {
            Identifier::Board(id) => {
                let mut stmt = conn.prepare(&format!("SELECT * FROM posts WHERE board=?1 ORDER BY time DESC LIMIT {} OFFSET ?2", crate::BOARD_PAGE_SIZE)).map_err(QueryError::SqlError)?;
                let rows = stmt.query_map([id.to_string(), (page * crate::BOARD_PAGE_SIZE).to_string()], Post::post_map(Identifier::Post)).map_err(QueryError::SqlError)?;
                let mut result = Vec::with_capacity(crate::BOARD_PAGE_SIZE as usize);
                for row in rows {
                    result.push(row.unwrap());
                }
                Ok(result)
            },
            Identifier::Post(_) => Err(QueryError::BadIdent(id)),
            Identifier::Reply(_) => Err(QueryError::BadIdent(id))
        }
    }

    pub fn get_replies(&self, conn : &rusqlite::Connection, count : i64) -> Result<Vec<Post>, QueryError> {
        match self.id {
            Identifier::Post(id) => {
                let mut stmt = conn.prepare("SELECT * FROM replies WHERE post=?1 ORDER BY time DESC LIMIT ?2").map_err(QueryError::SqlError)?;
                let rows = stmt.query_map([id.to_string(), count.to_string()], Post::post_map(Identifier::Reply)).map_err(QueryError::SqlError)?;
                let mut result = Vec::with_capacity(count as usize);
                for row in rows {
                    result.push(row.unwrap());
                }
                Ok(result)
            },
            Identifier::Board(_) => Err(QueryError::BadIdent(self.id)),
            Identifier::Reply(_) => Err(QueryError::BadIdent(self.id))
        }
    }

    pub fn get_all_replies(&self, conn : &rusqlite::Connection) -> Result<Vec<Post>, QueryError> {
        match self.id {
            Identifier::Post(id) => {
                let mut stmt = conn.prepare("SELECT * FROM replies WHERE post=?1 ORDER BY time DESC").map_err(QueryError::SqlError)?;
                let rows = stmt.query_map([id.to_string()], Post::post_map(Identifier::Reply)).map_err(QueryError::SqlError)?;
                let mut result = Vec::new();
                for row in rows {
                    result.push(row.unwrap());
                }
                Ok(result)
            },
            Identifier::Board(_) => Err(QueryError::BadIdent(self.id)),
            Identifier::Reply(_) => Err(QueryError::BadIdent(self.id))
        }
    }

    pub fn send(&self, conn : &rusqlite::Connection) -> Result<(), QueryError> {
        let mut stmt = match self.id {
            Identifier::Post(_) => {
                conn.prepare("INSERT INTO posts(username, content, board, image, time) VALUES(?1, ?2, ?3, ?4, ?5)").map_err(QueryError::SqlError)?
            },
            Identifier::Board(_) => { return Err(QueryError::BadIdent(self.id)); },
            Identifier::Reply(_) => {
                conn.prepare("INSERT INTO replies(username, content, post, image, time) VALUES(?1, ?2, ?3, ?4, ?5)").map_err(QueryError::SqlError)?
            }
        };
        stmt.execute([self.username.clone(), self.content.clone(), self.target.unwrap().to_string(), self.get_image_or_empty(), self.time.to_string()]).map_err(QueryError::SqlError)?;
        Ok(())
    }

    pub fn get_image_or_empty(&self) -> String {
        match &self.image {
            Some(image) => image.clone(),
            None => String::new()
        }
    }

    pub fn new_post(username : String, content : String, image : Option<String>, target : Identifier) -> Post {
        Post {
            username,
            content,
            image,
            target,
            id : Identifier::Post(-1),
            time: get_utc()
        }
    }

    pub fn new_reply(username : String, content : String, image : Option<String>, target : Identifier) -> Post {
        Post {
            username,
            content,
            image,
            target,
            id : Identifier::Reply(-1),
            time: get_utc()
        }
    }
}

impl Board {
    fn board_map(row : &rusqlite::Row<'_>) -> rusqlite::Result<Board> {
        Ok(Board {
            name : row.get(0)?,
            topic : row.get(1)?,
            id : Identifier::Board(row.get(2)?)
        })
    }

    pub fn get_all_boards(db : &rusqlite::Connection) -> Result<Vec<Board>, QueryError> {
        let mut stmt = db.prepare("SELECT * FROM boards").map_err(QueryError::SqlError)?;
        let rows = stmt.query_map([], Self::board_map).map_err(QueryError::SqlError)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.unwrap());
        }
        Ok(result)
    }

    pub fn get_one_board(db : &rusqlite::Connection, id : Identifier) -> Result<Board, QueryError> {
        let mut stmt = db.prepare("SELECT * FROM boards WHERE id=?1 LIMIT 1").map_err(QueryError::SqlError)?;
        let mut rows = stmt.query_map([id.unwrap().to_string()], Self::board_map).map_err(QueryError::SqlError)?;
        rows.next().ok_or(QueryError::NoResults)?.map_err(QueryError::SqlError)
    }

    pub fn get_by_name(db : &rusqlite::Connection, name : String) -> Result<Board, QueryError> {
        let mut stmt = db.prepare("SELECT * FROM boards WHERE name=?1 LIMIT 1").map_err(QueryError::SqlError)?;
        let mut rows = stmt.query_map([name], Self::board_map).map_err(QueryError::SqlError)?;
        rows.next().ok_or(QueryError::NoResults)?.map_err(QueryError::SqlError)
    }

    pub fn count_boards(db : &rusqlite::Connection) -> Result<usize, QueryError> {
        let mut stmt = db.prepare("SELECT COUNT(*) FROM boards").map_err(QueryError::SqlError)?;
        let mut rows = stmt.query([]).map_err(QueryError::SqlError)?;
        rows.next().map_err(QueryError::SqlError)?.ok_or(QueryError::NoResults)?.get(0).map_err(QueryError::SqlError)
    }
}

// check if the database exists; if not, create it
// warning: may panic

pub fn get_database(config : &Config) -> rusqlite::Connection {
    if !std::fs::exists(&config.database).unwrap() {
        let db = Connection::open(&config.database).unwrap();
        {
            let mut setup_boards = db.prepare("CREATE TABLE boards (name TEXT, desc TEXT, id INTEGER PRIMARY KEY)").unwrap();
            let mut setup_posts = db.prepare("CREATE TABLE posts (username TEXT, content TEXT, board INT, image TEXT, id INTEGER PRIMARY KEY, time INT)").unwrap();
            let mut setup_replies = db.prepare("CREATE TABLE replies (username TEXT, content TEXT, post INT, image TEXT, id INTEGER PRIMARY KEY, time INT)").unwrap();
            setup_boards.execute([]).unwrap();
            setup_posts.execute([]).unwrap();
            setup_replies.execute([]).unwrap();

            let mut create_board = db.prepare("INSERT INTO boards(name, desc) VALUES (?1, ?2)").unwrap();
            for board in &config.default_boards {
                create_board.execute([board.name.clone(), board.subject.clone()]).unwrap();
            }
        }
        db
    }
    else {
        Connection::open(&config.database).unwrap()
    }
}
