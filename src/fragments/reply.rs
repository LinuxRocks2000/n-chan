// actual post rendered on either a board or
// on a /post/ page
use maud::{ Markup, html };
use crate::queries::Post;
use crate::fragments;


pub fn reply(r : Post) -> Markup {
    html!{
        div class="reply" {
            @if let Some(image) = r.image {
                a href={"/img/" (image)} style={"background-image: url(\"/img/" (image) "\")" } {}
            }
            (fragments::user(r.username))
            br;
            (r.content)
        }
    }
}


