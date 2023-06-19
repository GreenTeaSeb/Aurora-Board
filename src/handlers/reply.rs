use super::{
    user::check_if_joined_board, user::get_by_id, user::get_user_boards, user::User,
    user::UserBoards, utils::sanitize_html,
};
use actix_identity::Identity;
use actix_web::{self, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use anyhow::Result;
use chrono::Utc;
use sailfish::TemplateOnce;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, MySqlPool};

#[derive(Deserialize)]
pub struct NewReplyData {
    text: String,
}

#[derive(Serialize, FromRow, Debug)]
pub struct Reply {
    pub id: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub poster_id: u32,
    pub parent_id: u32,
    pub text: String,
}

#[derive(Serialize, FromRow, Debug)]
pub struct ReplyData {
    pub id: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub poster_data: User,
    pub text: String,
    pub status: Option<bool>,
}

//DATABASE

async fn new_reply_db(
    id: u32,
    parent_id: String,
    form: &NewReplyData,
    pool: &MySqlPool,
) -> Result<u32> {
    let new = sqlx::query_as!(
        Post,
        r#"
insert into replies (poster_id,parent_id,text)
values(?,?,?);
        "#,
        id,
        parent_id,
        sanitize_html(&form.text)
    )
    .execute(pool)
    .await?;
    let last_id: u32 = new.last_insert_id() as u32;
    Ok(last_id)
}

pub async fn get_reply_by_id(parent_id: u32, pool: &MySqlPool) -> Result<Reply> {
    Ok(sqlx::query_as!(
        Reply,
        r#"
	SELECT replies.id, replies.created_at,replies.poster_id,replies.parent_id,replies.text FROM  replies
	WHERE replies.id = ? 
        "#,
        parent_id
    )
    .fetch_one(pool)
    .await?)
}

pub async fn get_n_replies(
    parent_id: u32,
    pool: &MySqlPool,
    limit: u32,
    offset: u32,
) -> Vec<Reply> {
    sqlx::query_as!(
        Reply,
        r#"
        SELECT replies.id, replies.created_at,replies.poster_id,replies.parent_id,replies.text FROM  replies
        WHERE replies.parent_id = ? 
        order by created_at asc
        limit ?
        offset ?
        "#,
        parent_id,
        limit,
        offset * limit 
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default()
}

async fn new_reply_like(
    id: u32,
    reply_id: u32,
    is_like: bool,
    pool: &MySqlPool,
) -> Result<sqlx::mysql::MySqlQueryResult, sqlx::Error> {
    sqlx::query!(
        r#"
insert into reply_likes (user_id,reply_id,is_like)
values (?,?,?)
on duplicate key update  is_like = ?
        "#,
        id,
        reply_id,
        is_like,
        is_like
    )
    .execute(pool)
    .await
}

pub async fn reply_status(id: u32, reply_id: u32, pool: &MySqlPool) -> Option<bool> {
    if let Ok(val) = sqlx::query!(
        r#"select is_like from reply_likes
    where user_id = ? and reply_id = ?;"#,
        id,
        reply_id
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

pub async fn new_reply(
    form: web::Form<NewReplyData>,
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
) -> impl Responder {
    let id = req.extensions().get::<u32>().unwrap().to_owned();
    let post = req
        .match_info()
        .get("post")
        .unwrap_or_default()
        .parse()
        .unwrap_or_default();

    match new_reply_db(id, post, &form.into_inner(), pool.as_ref()).await {
        Ok(newid) => HttpResponse::Ok().json(newid),
        Err(e) => HttpResponse::UnprocessableEntity().body(e.to_string()),
    }
}

pub async fn like_reply(pool: web::Data<MySqlPool>, req: HttpRequest) -> impl Responder {
    let id = req.extensions().get::<u32>().unwrap().to_owned();
    let reply_id = match req.match_info().get("reply") {
        Some(p) => p.to_string().parse::<u32>(),
        None => return HttpResponse::UnprocessableEntity().finish(),
    };
    match reply_id {
        Ok(replyid) => match new_reply_like(id, replyid, true, pool.as_ref()).await {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
        },
        Err(e) => return HttpResponse::UnprocessableEntity().body(e.to_string()),
    }
}
pub async fn dislike_reply(pool: web::Data<MySqlPool>, req: HttpRequest) -> impl Responder {
    let id = req.extensions().get::<u32>().unwrap().to_owned();
    let reply_id = match req.match_info().get("reply") {
        Some(p) => p.to_string().parse::<u32>(),
        None => return HttpResponse::UnprocessableEntity().finish(),
    };
    match reply_id {
        Ok(replyid) => match new_reply_like(id, replyid, false, pool.as_ref()).await {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
        },
        Err(e) => return HttpResponse::UnprocessableEntity().body(e.to_string()),
    }
}
