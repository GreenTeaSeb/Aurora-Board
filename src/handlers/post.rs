use super::user::check_if_joined_board;
use actix_web::{self, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, MySqlPool};

#[derive(Deserialize)]
pub struct NewPostdata {
    title: String,
    text: String,
}

#[derive(Serialize, FromRow, Debug)]
pub struct Post {
    pub id: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub poster_id: u32,
    pub board_id: u32,
    pub title: String,
    pub text: String,
}

//LOGIC

async fn new_post(id: u32, board_name: String, form: NewPostdata, pool: &MySqlPool) -> Result<u32> {
    let new = sqlx::query_as!(
        Post,
        r#"
insert into posts (poster_id,board_id,title,text)
values(?, (select id from boards where name = ?) ,?,?);
    "#,
        id,
        board_name,
        form.title,
        form.text
    )
    .execute(pool)
    .await?;
    let last_id: u32 = new.last_insert_id() as u32;
    Ok(last_id)
}

async fn new_like(
    id: u32,
    post_id: u32,
    is_like: bool,
    pool: &MySqlPool,
) -> Result<sqlx::mysql::MySqlQueryResult, sqlx::Error> {
    sqlx::query!(
        r#"
insert into likes (user_id,post_id,is_like)
values (?,?,?)
on duplicate key update  is_like = ?
        "#,
        id,
        post_id,
        is_like,
        is_like
    )
    .execute(pool)
    .await
}

pub async fn is_liked(id: u32, post_id: u32, pool: &MySqlPool) -> Option<bool> {
    if let Ok(val) = sqlx::query!(
        r#"select is_like from likes
where user_id = ? and post_id = ?;"#,
        id,
        post_id
    )
    .fetch_one(pool)
    .await
    {
        Some(val.is_like != 0)
    } else {
        None
    }
}

//API
pub async fn newpost(
    form: web::Form<NewPostdata>,
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
) -> impl Responder {
    let id = req.extensions().get::<u32>().unwrap().to_owned();
    let board = req.match_info().get("name").ok_or("").unwrap_or_default();
    if !check_if_joined_board(id, board, pool.get_ref())
        .await
        .unwrap_or_default()
    {
        return HttpResponse::Unauthorized().body("User not in board");
    }
    match new_post(id, board.to_string(), form.into_inner(), pool.as_ref()).await {
        Ok(newid) => HttpResponse::Found()
            .append_header(("location", format!("/boards/{}", board)))
            .json(newid),
        Err(e) => HttpResponse::UnprocessableEntity().body(e.to_string()),
    }
}

pub fn time_msg(old: chrono::DateTime<Utc>) -> String {
    let new = Utc::now();
    let diff = new.signed_duration_since(old);
    let mins = diff.num_minutes();
    match mins {
        0 => String::from("a few seconds ago"),
        1 => format!("{} minute ago", mins),
        2..=59 => format!("{} minutes ago", mins),
        60..=119 => format!("{} hour ago", diff.num_hours()),
        120..=1439 => format!("{} hours ago", diff.num_hours()),
        _ => old.format("%x").to_string(),
    }
}

pub async fn like_post(pool: web::Data<MySqlPool>, req: HttpRequest) -> impl Responder {
    let id = req.extensions().get::<u32>().unwrap().to_owned();
    let post_id = match req.match_info().get("post") {
        Some(p) => p.to_string().parse::<u32>(),
        None => return HttpResponse::UnprocessableEntity().finish(),
    };
    match post_id {
        Ok(postid) => match new_like(id, postid, true, pool.as_ref()).await {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
        },
        Err(e) => return HttpResponse::UnprocessableEntity().body(e.to_string()),
    }
}
pub async fn dislike_post(pool: web::Data<MySqlPool>, req: HttpRequest) -> impl Responder {
    let id = req.extensions().get::<u32>().unwrap().to_owned();
    let post_id = match req.match_info().get("post") {
        Some(p) => p.to_string().parse::<u32>(),
        None => return HttpResponse::UnprocessableEntity().finish(),
    };
    match post_id {
        Ok(postid) => match new_like(id, postid, false, pool.as_ref()).await {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
        },
        Err(e) => return HttpResponse::UnprocessableEntity().body(e.to_string()),
    }
}
