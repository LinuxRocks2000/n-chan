use crate::Config;
use maud::{Markup, html, DOCTYPE};
use crate::RenderError;


pub async fn pageroot(db : &crate::WeldsClient, config : &Config, content : impl maud::Render) -> Result<Markup, Box<dyn std::error::Error>> {
    Ok(html! {
        (DOCTYPE)
        html {
            head {
                title { "N-CHAN: THE BEST IMAGEBOARD" }
                link rel="stylesheet" href="/res/main.css";
                link rel="icon" href={"/res/" (config.icon.clone()) };
            }
            body {
                (crate::fragments::top(db, config.banner.clone()).await?)
                (content)
                script src="/res/main.js" {}
            }
        }
    })
}

