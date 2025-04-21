// ORM definitions
use welds::{ connections, WeldsModel, prelude::* };
pub const BOARD_PAGE_SIZE : i64 = 50;


#[derive(WeldsModel, Debug, Clone)]
#[welds(table="posts")]
pub struct Post { // a post
    pub username : String,
    pub content : String,
    pub image : String, // a filename
    #[welds(primary_key)]
    pub id : i64,
    pub board : i64,
    pub time : i64
}


#[derive(WeldsModel, Debug, Clone)]
#[welds(table="replies")]
pub struct Reply { // a reply
    pub username : String,
    pub content : String,
    pub image : Option<String>, // a filename
    #[welds(primary_key)]
    pub id : i64,
    pub post : i64,
    pub time : i64
}


#[derive(WeldsModel, Debug, Clone)]
#[welds(table="boards")]
pub struct Board {
    pub name : String, // a few letters/numbers with no spaces. pretty printed in forward slashes. ex; /pol/
    pub desc : String, // board description
    #[welds(primary_key)]
    pub id : i64,
    pub minacc : i32
}


#[derive(WeldsModel, Debug, Clone)]
#[welds(table="users")]
pub struct User {
    #[welds(primary_key)]
    pub id : i64,
    pub username : String,
    pub password : String, // sha256 hashed
    pub rights : i32 // 0 = no access (this is the default permission level for anonymous users)
                     // > 0 = no special perms, can access logged-in-only boards
                     // > 1 = update or delete users with lower perms than you
                     // > 10 = delete posts (cannot delete posts by users with a >= permission level)
                     // > 20 = create/update/delete boards (cannot create boards with a higher minacc than your permissions)
                     // > 30 = update website (this includes changing header links, news, homepage, etc)
                     // by default, logged in users have a permission int of 1.
                     // note that logging in doesn't force you to use your official username! however, when you do,
                     // it shows a little "verified" sticker with your permission level

                     // at user creation, if there is no user id 0 (administrator), the first user created becomes administrator
                     // with permission int 100.
}


impl Board {
    pub async fn get_by_name(welds : &crate::WeldsClient, name : String) -> Result<Board, Box<dyn std::error::Error>> {
        let query = Board::all().limit(1).where_col(move |x| x.name.equal(name.clone())).run(welds).await?;
        let board = query.first().ok_or("no such board")?;
        Ok(board.as_ref().clone())
    }

    pub async fn get_one_board(welds : &crate::WeldsClient, id : i64) -> Result<Board, Box<dyn std::error::Error>> {
        Ok(Board::all().limit(1).where_col(|x| x.id.equal(id)).run(welds).await?.first().ok_or("no such board")?.as_ref().clone())
    }

    pub async fn count_boards(welds : &crate::WeldsClient) -> Result<i64, Box<dyn std::error::Error>> {
        let rows = welds.fetch_rows("SELECT COUNT(*) FROM boards;", &[]).await?;
        let first = rows.first().unwrap();
        Ok(first.get_by_position(0)?)
    }

    pub async fn get_all_boards(welds : &crate::WeldsClient) -> Result<Vec<Board>, Box<dyn std::error::Error>> {
        Ok(Board::all().run(welds).await?.into_inners())
    }
}

impl Post {
    pub async fn get_from_board(welds : &crate::WeldsClient, board : i64, page : i64) -> Result<Vec<Post>, Box<dyn std::error::Error>> {
        Ok(Post::all().limit(BOARD_PAGE_SIZE).offset(BOARD_PAGE_SIZE * page).where_col(|x| x.board.equal(board)).run(welds).await?.into_inners())
    }

    pub async fn get_one_post(welds : &crate::WeldsClient, id : i64) -> Result<Post, Box<dyn std::error::Error>> {
        Ok(Post::all().limit(1).where_col(|x| x.id.equal(id)).run(welds).await?.first().ok_or("no such post")?.as_ref().clone())
    }

    pub async fn get_replies(&self, welds : &crate::WeldsClient, paginate : Option<(i64, i64)>) -> Result<Vec<Reply>, Box<dyn std::error::Error>> {
        let mut query = Reply::all();
        if let Some((page_size, page)) = paginate {
            query = query.limit(page_size).offset(page_size * page);
        }
        Ok(query.where_col(|x| x.post.equal(self.id)).run(welds).await?.into_inners())
    }

    pub fn new_post(username : String, content : String, image : String, board : i64) -> DbState<Post> {
        let mut ret = Post::new();
        ret.username = username;
        ret.content = content;
        ret.image = image;
        ret.board = board;
        ret
    }
}

impl Reply {
    pub fn new_reply(username : String, content : String, image : Option<String>, post : i64) -> DbState<Reply> {
        let mut reply = Reply::new();
        reply.username = username;
        reply.content = content;
        reply.image = image;
        reply.post = post;
        reply
    }
}