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

#[derive(Serialize, FromRow, Debug)]
pub struct PostData {
    pub id: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub poster_id: u32,
    pub poster_pfp: String,
    pub title: String,
    pub text: String,
}

#[derive(TemplateOnce)]
#[template(path = "post.html", escape = false)]
struct PostTemplate {
    user: Result<User>,
    user_boards: Vec<UserBoards>,
    post: PostData,
    status: Option<bool>
}

//LOGIC

async fn new_post(
    id: u32,
    board_name: String,
    form: &NewPostdata,
    pool: &MySqlPool,
) -> Result<u32> {
    let new = sqlx::query_as!(
        Post,
        r#"
insert into posts (poster_id,board_id,title,text)
values(?, (select id from boards where name = ?) ,?,?);
    "#,
        id,
        board_name,
        form.title,
        sanitize_html(&form.text)
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

pub async fn get_post_by_id(id: u32, pool: &MySqlPool) -> Result<PostData> {
    Ok(sqlx::query_as!(
        PostData,
        r#"
	SELECT posts.id, posts.created_at,posts.poster_id,posts.title,posts.text, users.pfp as poster_pfp FROM  posts
        INNER JOIN users ON users.id = posts.poster_id
	WHERE posts.id = ? 
        "#,
        id
    )
    .fetch_one(pool)
    .await?)
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
    match new_post(id, board.to_string(), &form.into_inner(), pool.as_ref()).await {
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

//FRONTEND

pub async fn post_pg(id: Identity, req: HttpRequest, pool: web::Data<MySqlPool>) -> impl Responder {
    let post = req
        .match_info()
        .get("post")
        .unwrap_or_default()
        .parse()
        .unwrap_or_default();
    let id: u32 = id
        .identity()
        .unwrap_or_default()
        .parse()
        .unwrap_or_default();

    let user_data = get_by_id(&id, pool.get_ref()).await;

    let post_data = match get_post_by_id(post, pool.get_ref()).await {
        Ok(d) => d,
        Err(x) => return HttpResponse::NotFound().body(x.to_string()),
    };

    let temp = PostTemplate {
        user: user_data,
        user_boards: get_user_boards(id, pool.get_ref()).await,
        post: post_data,
        status: is_liked(id, post, pool.get_ref()).await
    };
    HttpResponse::Found().body(temp.render_once().unwrap())
}
