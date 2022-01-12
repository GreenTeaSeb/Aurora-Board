use crate::handlers::user::check_if_logged_in;
use crate::handlers::user::{self, User};
use actix_files as fs;
use actix_identity::Identity;
use actix_web::{self, get, post, web, HttpRequest, HttpResponse, Responder};
use anyhow::{anyhow, Result};
use sailfish::TemplateOnce;
use serde::{Deserialize, Serialize};
use sqlx::{types::chrono, FromRow, MySqlPool};

use super::user::check_if_joined_board;

#[derive(Serialize, FromRow, Debug)]
pub struct Board {
    id: u32,
    owner_id: u32,
    name: String,
    description: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
pub struct NewBoardData {
    name: String,
    desc: String,
}

#[get("/board")]
pub async fn newboard_pg(id: Identity) -> std::io::Result<fs::NamedFile> {
    if id.identity().is_none() {
        return fs::NamedFile::open("./static/login.html");
    };
    Ok(fs::NamedFile::open("./static/newboard.html")?)
}

async fn create_board(id: u64, form: NewBoardData, pool: &MySqlPool) -> Result<String> {
    let boards = sqlx::query_as!(
        Board,
        r#"
	SELECT * FROM boards
	WHERE name = ?
    "#,
        form.name
    )
    .fetch_all(pool)
    .await?;
    if boards.is_empty() {
        sqlx::query!(
            r#"
            INSERT INTO boards (name, description, owner_id)
	    VALUES (?,?,?)
	"#,
            form.name,
            form.desc,
            id
        )
        .execute(pool)
        .await?;
        Ok(form.name)
    } else {
        return Err(anyhow!("Board already exists, try another name"));
    }
}

#[post("/board")]
pub async fn newboard(
    id: Identity,
    form: web::Form<NewBoardData>,
    pool: web::Data<MySqlPool>,
) -> impl Responder {
    let id_int = match check_if_logged_in(id.identity()) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::Found()
                .append_header(("location", "/login"))
                .finish()
        }
    };
    match create_board(id_int, form.into_inner(), pool.as_ref()).await {
        Ok(id) => HttpResponse::Found()
            .append_header(("location", format!("/b/{}", id)))
            .finish(),
        Err(e) => HttpResponse::UnprocessableEntity().body(e.to_string()),
    }
}

pub async fn get_by_name(name: &str, pool: &MySqlPool) -> Result<Board> {
    Ok(sqlx::query_as!(
        Board,
        r#"   
	SELECT * FROM boards
	WHERE name = ? 
    "#,
        name
    )
    .fetch_one(pool)
    .await?)
}

#[derive(TemplateOnce)]
#[template(path = "board.stpl", escape = false)]
struct BoardTemplate {
    board: Board,
    user: anyhow::Result<User>,
    is_in: bool,
}

#[get("{name}")]
pub async fn board_pg(id: Identity, req: HttpRequest, pool: web::Data<MySqlPool>) -> HttpResponse {
    let board = req.match_info().get("name").ok_or("").unwrap_or_default();
    let user_id_int: u64 = id
        .identity()
        .unwrap_or_default()
        .parse()
        .unwrap_or_default();
    let user_data = user::get_by_id(&user_id_int, pool.get_ref()).await;
    let board_data = get_by_name(board, pool.get_ref()).await;
    match board_data {
        Ok(b) => {
            let temp = BoardTemplate {
                user: user_data,
                board: b,
                is_in: check_if_joined_board(user_id_int, board, pool.get_ref())
                    .await
                    .unwrap_or_default(),
            };
            HttpResponse::Ok().body(temp.render_once().unwrap())
        }
        Err(_) => HttpResponse::NotFound().body("Board doesn't exist"),
    }
}

async fn user_join(id: &u64, board: &str, pool: &MySqlPool) -> Result<()> {
    sqlx::query!(
        r#"
	INSERT INTO members (user_id, board_id, admin)
VALUES (?, (select id from boards where name = ? ) , ?)
	"#,
        id,
        board,
        false
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn user_leave(id: &u64, board: &str, pool: &MySqlPool) -> Result<()> {
    sqlx::query!(
        r#"
DELETE FROM members where user_id = ? and  (select id from boards where name = ? ) 
	"#,
        id,
        board
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[post("{name}/join")]
pub async fn join_board(
    id: Identity,
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
) -> impl Responder {
    let id = match check_if_logged_in(id.identity()) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::Found()
                .append_header(("location", "/login"))
                .finish()
        }
    };

    let board = req.match_info().get("name").ok_or("").unwrap_or_default();
    if get_by_name(board, pool.get_ref()).await.is_ok() {
        user_join(&id, board, pool.get_ref()).await;
        return HttpResponse::Found()
            .append_header(("location", format!("/b/{}", board)))
            .finish();
    }

    HttpResponse::NotFound().body("Board doesn't exist")
}

#[post("{name}/leave")]
pub async fn leave_board(
    id: Identity,
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
) -> impl Responder {
    let id = match check_if_logged_in(id.identity()) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::Found()
                .append_header(("location", "/login"))
                .finish()
        }
    };

    let board = req.match_info().get("name").ok_or("").unwrap_or_default();
    if get_by_name(board, pool.get_ref()).await.is_ok() {
        user_leave(&id, board, pool.get_ref()).await;
        return HttpResponse::Found()
            .append_header(("location", format!("/b/{}", board)))
            .finish();
    }

    HttpResponse::NotFound().body("Board doesn't exist")
}
