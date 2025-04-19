use maud::{ Markup, html };
use crate::queries::Board;
use crate::RenderError;

pub fn top(db : &rusqlite::Connection, banner : String) -> Result<Markup, RenderError> {
    let boards = Board::get_all_boards(db).map_err(RenderError::QueryFailure)?;
    Ok(html!{
        div id="top" {
            a href="/" { img src={ "/res/" (banner)}; }
            p {
                @for o_b in boards {
                    a href={ "/b/" (o_b.id.unwrap()) } { "/" (o_b.name) "/" }
                }
            }
        }
    })
}
