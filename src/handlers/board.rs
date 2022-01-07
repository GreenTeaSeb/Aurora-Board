use crate::handlers::user::{self, User};
use actix_files as fs;
use actix_identity::Identity;
use actix_web::{self, get, post, web, HttpRequest, HttpResponse, Responder};
use anyhow::{anyhow, Result};
use sailfish::TemplateOnce;
use serde::{Deserialize, Serialize};
use sqlx::{types::chrono, FromRow, MySqlPool};
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

#[get("/newboard")]
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
        let new = sqlx::query!(
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

#[post("/newboard")]
pub async fn newboard(
    id: Identity,
    form: web::Form<NewBoardData>,
    pool: web::Data<MySqlPool>,
) -> impl Responder {
    let id_int: u64 = id
        .identity()
        .unwrap_or_default()
        .parse()
        .unwrap_or_default();

    if id.identity().is_none() {
        return HttpResponse::UnprocessableEntity().body("Not logged in");
    };

    match create_board(id_int, form.into_inner(), pool.as_ref()).await {
        Ok(id) => HttpResponse::Found()
            .append_header(("location", format!("/board/{}", id)))
            .finish(),
        Err(e) => HttpResponse::UnprocessableEntity().body(e.to_string()),
    }
}

#[derive(TemplateOnce)]
#[template(path = "board.stpl", escape = false)]
struct BoardTemplate {
    board: Board,
    user: anyhow::Result<User>,
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

#[get("/board/{name}")]
pub async fn board_pg(
    user_id: Identity,
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
) -> HttpResponse {
    let id = req.match_info().get("name").ok_or("").unwrap_or_default();
    let user_id_int: u64 = user_id
        .identity()
        .unwrap_or_default()
        .parse()
        .unwrap_or_default();
    let user = user::get_by_id(&user_id_int, pool.get_ref()).await;
    let board = get_by_name(&id, pool.get_ref()).await;
    match board {
        Ok(b) => {
            let temp = BoardTemplate {
                user: user,
                board: b,
            };
            HttpResponse::Ok().body(temp.render_once().unwrap())
        }
        Err(_) => HttpResponse::NotFound().body("Board doesn't exist"),
    }
}
