// a standardized "post" box that calls out to an api
use maud::{ Markup, html };
use crate::queries::Identifier;


pub fn post_box(action : &str, class : &str) -> Markup {
    html! {
        form action = { (action) } method="post" enctype="multipart/form-data" onsubmit="before_submit()" class={ "postbox " (class) } {
            span { "username: " input type="text" name="username" class="username"; }
            span { textarea name="content" {} }
            span { "image: " input type="file" name="image"; }
            span { input type="submit" value="Post"; }
        }
    }
}
