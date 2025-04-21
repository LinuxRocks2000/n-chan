// username text
use maud::{ Markup, html };

pub fn user(name : String) -> Markup {
    html! {
        span class="user" {(name)}
    }
}
