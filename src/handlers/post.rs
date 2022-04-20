use super::user::check_if_joined_board;
use actix_web::{self, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, MySqlPool};
use chrono::Utc;

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

pub fn time_msg(old: chrono::DateTime<Utc>) -> String{
    let new = Utc::now();
   let diff =  new.signed_duration_since(old);
   let mins = diff.num_minutes();
   match mins {
      0 => return String::from("a few seconds ago"),
      1 => return format!("{} minute ago",mins),
      2..=59 => return format!("{} minutes ago",mins),
      60..=119 => return format!("{} hour ago", diff.num_hours()),
      120..=1439 => return format!("{} hours ago", diff.num_hours()),
      _ => return old.format("%x").to_string()
   }
}


