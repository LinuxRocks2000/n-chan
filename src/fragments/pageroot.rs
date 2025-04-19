use rusqlite::Connection;
use crate::Config;
use maud::{Markup, html, DOCTYPE};
use crate::RenderError;


pub fn pageroot(db : &Connection, config : &Config, content : impl maud::Render) -> Result<Markup, RenderError> {
    Ok(html! {
        (DOCTYPE)
        html {
            head {
                title { "N-CHAN: THE BEST IMAGEBOARD" }
                link rel="stylesheet" href="/res/main.css";
                link rel="icon" href={"/res/" (config.icon.clone()) };
            }
            body {
                (crate::fragments::top(db, config.banner.clone())?)
                (content)
                script src="/res/main.js" {}
            }
        }
    })
}

