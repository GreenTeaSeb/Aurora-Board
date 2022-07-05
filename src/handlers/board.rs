use super::post::Post;
use super::user::{check_if_joined_board, check_if_owner, get_user_boards, UserBoards};
use crate::handlers::user::{self, User};
use actix_files as fs;
use actix_identity::Identity;
use actix_multipart::Multipart;
use actix_web::{
    self,
    web::{self, Query},
    HttpMessage, HttpRequest, HttpResponse, Responder,
};
use anyhow::{anyhow, Context, Result};
use futures_util::TryStreamExt as _;
use sailfish::TemplateOnce;
use serde::{Deserialize, Serialize};
use sqlx::{types::chrono, FromRow, MySqlPool};
use std::env;
use std::io::Write;

#[derive(Serialize, FromRow, Debug)]
pub struct Board {
    id: u32,
    owner_id: u32,
    name: String,
    description: String,
    created_at: chrono::DateTime<chrono::Utc>,
    icon: String,
}

#[derive(Deserialize)]
pub struct NewBoardData {
    name: String,
    desc: String,
}

#[derive(TemplateOnce)]
#[template(path = "board.stpl", escape = false)]
struct BoardTemplate {
    board: Board,
    user: anyhow::Result<User>,
    is_in: bool,
    is_owner: bool,
    user_boards: Vec<UserBoards>,
    posts: Vec<Post>,
}

#[derive(Deserialize, Debug)]
pub struct BoardQuery {
    page: Option<u32>,
}

#[derive(Deserialize, Debug)]
pub struct GetPostsQuery {
    limit: u32,
    offset: u32,
}
// LOGIC FUNCTIONS

async fn create_board(id: u32, form: NewBoardData, pool: &MySqlPool) -> Result<String> {
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

async fn user_join(id: &u32, board: &str, pool: &MySqlPool) -> Result<()> {
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

async fn user_leave(id: &u32, board: &str, pool: &MySqlPool) -> Result<()> {
    sqlx::query!(
        r#"
DELETE FROM members where user_id = ? and board_id = (select id from boards where name = ? )

	"#,
        id,
        board
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn delete_board_db(board: &str, pool: &MySqlPool) -> Result<()> {
    sqlx::query!(
        r#"
DELETE FROM boards where name = ? ;
	"#,
        board
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_n_posts(
    board_name: String,
    pool: &MySqlPool,
    limit: u32,
    offset: u32,
) -> Vec<Post> {
    sqlx::query_as!(
        Post,
        r#"
select * from posts
where board_id = (select id from boards where name = ?)
order by created_at desc
limit ?
offset ?
"#,
        board_name,
        limit,
        offset * limit
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default()
}

async fn save_icon(mut payload: Multipart, board: String) -> Result<String> {
    while let Some(mut field) = payload.try_next().await? {
        let extension = field
            .content_disposition()
            .get_filename()
            .context("Missing file name!")?
            .split(".")
            .last()
            .context("Missing file type")?;
        let name = format!("{}.{}", board, extension);
        let data_dir = env::var("DATA")?;
        let path = format!("{}/{}", data_dir, name);
        let mut file = web::block(|| std::fs::File::create(path)).await??;
        while let Some(chunk) = field.try_next().await? {
            file = web::block(move || file.write_all(&chunk).map(|_| file)).await??;
        }
        return Ok(name);
    }
    anyhow::bail!("No  payload");
}

async fn set_icon(board: String, file_name: String, pool: &MySqlPool) -> Result<()> {
    sqlx::query!(
        r#"
update boards
set icon = ? 
where name = ?
        "#,
        format!("/data/{}", file_name),
        board
    )
    .execute(pool)
    .await?;
    Ok(())
}

// FRONTEND

pub async fn newboard_pg() -> std::io::Result<fs::NamedFile> {
    Ok(fs::NamedFile::open("./static/newboard.html")?)
}

pub async fn board_pg(
    id: Identity,
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    q: Query<BoardQuery>,
) -> impl Responder {
    let board = req.match_info().get("name").ok_or("").unwrap_or_default();
    let id: u32 = id
        .identity()
        .unwrap_or_default()
        .parse()
        .unwrap_or_default();
    let user_data = user::get_by_id(&id, pool.get_ref()).await;
    let board_data = get_by_name(board, pool.get_ref()).await;
    let offset = q.page.unwrap_or_default();
    let isin = check_if_joined_board(id, board, pool.get_ref())
        .await
        .unwrap_or_default();

    match board_data {
        Ok(b) => {
            let temp = BoardTemplate {
                user: user_data,
                board: b,
                is_in: isin,
                is_owner: check_if_owner(id, board, pool.get_ref())
                    .await
                    .unwrap_or_default(),
                user_boards: get_user_boards(id, pool.get_ref()).await,
                posts: get_n_posts(board.to_string(), pool.get_ref(), 10, offset).await,
            };
            HttpResponse::Found().body(temp.render_once().unwrap())
        }
        Err(_) => HttpResponse::NotFound().body("Board doesn't exist"),
    }
}

// API

pub async fn join_board(pool: web::Data<MySqlPool>, req: HttpRequest) -> impl Responder {
    let id = req.extensions().get::<u32>().unwrap().to_owned();
    let board = req.match_info().get("name").ok_or("").unwrap_or_default();
    if !get_by_name(board, pool.get_ref()).await.is_ok() {
        return HttpResponse::NotFound().body("Board doesn't exist");
    }

    if check_if_owner(id, board, pool.get_ref())
        .await
        .unwrap_or_default()
    {
        return HttpResponse::Unauthorized().body("Owner can't join own board");
    }
    if check_if_joined_board(id, board, pool.get_ref())
        .await
        .unwrap_or_default()
    {
        return HttpResponse::Found()
            .append_header(("location", format!("/boards/{}", board)))
            .body("User is already in server");
    }
    if !user_join(&id, board, pool.get_ref()).await.is_ok() {
        return HttpResponse::InternalServerError().body("Failed to join");
    }
    return HttpResponse::Found()
        .append_header(("location", format!("/boards/{}", board)))
        .finish();
}

pub async fn leave_board(pool: web::Data<MySqlPool>, req: HttpRequest) -> impl Responder {
    let id = req.extensions().get::<u32>().unwrap().to_owned();
    let board = req.match_info().get("name").ok_or("").unwrap_or_default();

    if check_if_owner(id, board, pool.get_ref())
        .await
        .unwrap_or_default()
    {
        return HttpResponse::Unauthorized().body("Owner can't leave");
    }
    if get_by_name(board, pool.get_ref()).await.is_err() {
        return HttpResponse::NotFound().body("Board doesn't exist");
    }
    if user_leave(&id, board, pool.get_ref()).await.is_ok() {
        return HttpResponse::Found()
            .append_header(("location", format!("/boards/{}", board)))
            .finish();
    }
    return HttpResponse::NotFound().body("Failed to leave board");
}

pub async fn delete_board(pool: web::Data<MySqlPool>, req: HttpRequest) -> impl Responder {
    let id = req.extensions().get::<u32>().unwrap().to_owned();
    let board = req.match_info().get("name").ok_or("").unwrap_or_default();

    if !check_if_owner(id, board, pool.get_ref())
        .await
        .unwrap_or_default()
    {
        return HttpResponse::Unauthorized().body("User can not delete board");
    }
    if get_by_name(board, pool.get_ref()).await.is_err() {
        return HttpResponse::NotFound().body("Board doesn't exist");
    }

    if let Err(x) = delete_board_db(board, pool.get_ref()).await {
        return HttpResponse::NotFound()
            .body("Failed to delete board: ".to_owned() + &x.to_string());
    }

    return HttpResponse::Found()
        .append_header(("location", "/"))
        .finish();
}
pub async fn newboard(
    form: web::Form<NewBoardData>,
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
) -> impl Responder {
    let id = req.extensions().get::<u32>().unwrap().to_owned();
    match create_board(id, form.into_inner(), pool.as_ref()).await {
        Ok(board_id) => {
            user_join(&id, &board_id, pool.as_ref()).await;
            HttpResponse::Found()
                .append_header(("location", format!("/boards/{}", board_id)))
                .json(board_id)
        }
        Err(e) => HttpResponse::UnprocessableEntity().body(e.to_string()),
    }
}

pub async fn get_posts(
    req: HttpRequest,
    pool: web::Data<MySqlPool>,
    q: Query<GetPostsQuery>,
) -> impl Responder {
    let board = req.match_info().get("name").ok_or("").unwrap_or_default();
    HttpResponse::Found()
        .json(get_n_posts(board.to_string(), pool.get_ref(), q.limit, q.offset).await)
}

pub async fn new_icon(
    req: HttpRequest,
    mut payload: Multipart,
    pool: web::Data<MySqlPool>,
) -> impl Responder {
    let id = req.extensions().get::<u32>().unwrap().to_owned();
    let board = req.match_info().get("name").ok_or("").unwrap_or_default();
    if let Ok(res) = check_if_owner(id, board, pool.get_ref()).await {
        if res {
            if let Ok(name) = save_icon(payload, board.to_string()).await {
                set_icon(board.to_string(), name, pool.get_ref()).await;
            }
        }
    }

    HttpResponse::SeeOther()
        .append_header(("location", format!("/boards/{}", board)))
        .finish()
}
