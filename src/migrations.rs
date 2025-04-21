// Welds migrations

use welds::errors::Result;
use welds::migrations::prelude::*;

fn m20250421_initial_create_boards(_state : &TableState) -> Result<MigrationStep> {
    let m = create_table("boards")
        .id(|c| c("id", Type::IntBig))
        .column(|c| c("name", Type::String))
        .column(|c| c("desc", Type::String));
    Ok(MigrationStep::new("m20250421_initial_create_boards", m))
}

fn m20250421_initial_create_posts(_state : &TableState) -> Result<MigrationStep> {
    let m = create_table("posts")
        .id(|c| c("id", Type::IntBig))
        .column(|c| c("username", Type::String))
        .column(|c| c("content", Type::String))
        .column(|c| c("image", Type::String))
        .column(|c| c("time", Type::IntBig))
        .column(|c| c("board", Type::IntBig));
    Ok(MigrationStep::new("m20250421_initial_create_posts", m))
}

fn m20250421_initial_create_replies(_state : &TableState) -> Result<MigrationStep> {
    let m = create_table("replies")
        .id(|c| c("id", Type::IntBig))
        .column(|c| c("username", Type::String))
        .column(|c| c("content", Type::String))
        .column(|c| c("image", Type::String))
        .column(|c| c("time", Type::IntBig))
        .column(|c| c("post", Type::IntBig));
    Ok(MigrationStep::new("m20250421_initial_create_replies", m))
}

fn m20250421_create_users(_state : &TableState) -> Result<MigrationStep> {
    let m = create_table("users")
        .id(|c| c("id", Type::IntBig))
        .column(|c| c("username", Type::String))
        .column(|c| c("password", Type::String))
        .column(|c| c("rights", Type::Int));
    Ok(MigrationStep::new("m20250421_create_users", m))
}

fn m20250421_add_minacc(state : &TableState) -> Result<MigrationStep> {
    let m = change_table(state, "users")?.add_column("minacc", Type::Int);
    Ok(MigrationStep::new("m20250421_add_minacc", m))
}

pub async fn up(client: &dyn welds::TransactStart) -> Result<()> {
    let list: Vec<MigrationFn> = vec![
        m20250421_initial_create_boards,
        m20250421_initial_create_posts,
        m20250421_initial_create_replies,
        m20250421_create_users,
        m20250421_add_minacc
    ];
    welds::migrations::up(client, list.as_slice()).await?;
    Ok(())
}