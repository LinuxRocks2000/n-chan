// actual post rendered on either a board or
// on a /post/ page
use maud::{ Markup, html };
use crate::queries::Post;
use crate::fragments;


pub fn post(p : Post, inner : Markup, location : i64, is_post_view : bool) -> Markup { // is_post_view: are we in a post page? if not, we're in a board
    html!{
        div class="post" {
            a href={"/img/" (p.image)} style={"background-image: url(\"/img/" (p.image) "\")" } {}
            (fragments::user(p.username))
            br;
            (p.content)
            br; br;
            (fragments::post_box(&format!("/reply/{}/{}/{}", p.id, if is_post_view { "post" } else { "board" }, location), "replyform"))
            (inner)
        }
    }
}

