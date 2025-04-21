// actual post rendered on either a board or
// on a /post/ page
use maud::{ Markup, html };
use crate::queries::Post;
use crate::fragments;
use crate::queries::Identifier;


pub fn post(p : Post, inner : Markup, where_am_i : Identifier) -> Markup {
    html!{
        div class="post" {
            @if let Some(image) = p.image {
                a href={"/img/" (image)} style={"background-image: url(\"/img/" (image) "\")" } {}
            }
            (fragments::user(p.username))
            br;
            (p.content)
            br; br;
            (fragments::post_box(&format!("/reply/{}/{}/{}", p.id.unwrap(), match where_am_i {
                Identifier::Board(_) => "board",
                Identifier::Post(_) => "post",
                _ => ""
            }, match where_am_i {
                Identifier::Board(_) => p.target.unwrap(),
                Identifier::Post(_) => p.id.unwrap(),
                _ => 0
            }), "replyform"))
            (inner)
        }
    }
}

