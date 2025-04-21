use maud::{ Markup, html };
use crate::queries::Board;
use crate::RenderError;

pub async fn top(db : &crate::WeldsClient, banner : String) -> Result<Markup, Box<dyn std::error::Error>> {
    let boards = Board::get_all_boards(db).await?;
    Ok(html!{
        div id="top" {
            a href="/" { img src={ "/res/" (banner)}; }
            p {
                @for o_b in boards {
                    a href={ "/b/" (o_b.id) } { "/" (o_b.name) "/" }
                }
            }
        }
    })
}
