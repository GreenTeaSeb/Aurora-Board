use actix_identity::Identity;
use actix_web::{get, web, HttpRequest, HttpResponse};
use anyhow::Result;
use sailfish::TemplateOnce;
use serde::Serialize;
use sqlx::{FromRow, MySqlPool};

#[derive(Serialize, FromRow, Debug)]
pub struct User {
    pub id: u32,
    pub username: String,
    pub email: String,
    pub password: Vec<u8>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub pfp: String,
    pub banner: String,
    pub bio: Option<String>,
}

#[derive(TemplateOnce)]
#[template(path = "user.stpl", escape = false)]
struct UserTemplate {
    user: anyhow::Result<User>,
    user_guest: User,
}

pub async fn get_by_id(id: &u64, pool: &MySqlPool) -> Result<User> {
    Ok(sqlx::query_as!(
        User,
        r#"
	SELECT * FROM users
	WHERE id = ? 
    "#,
        id
    )
    .fetch_one(pool)
    .await?)
}

#[get("/user/{id}")]
pub async fn get(id: Identity, req: HttpRequest, pool: web::Data<MySqlPool>) -> HttpResponse {
    let id_guest = req.match_info().get("id").ok_or("no such id");
    let id_guest_int: u64 = id_guest.unwrap_or_default().parse().unwrap_or_default();
    let user_guest_data = get_by_id(&id_guest_int, pool.get_ref()).await;

    let id_int: u64 = id
        .identity()
        .unwrap_or_default()
        .parse()
        .unwrap_or_default();
    let user_data = get_by_id(&id_int, pool.get_ref()).await;

    match user_guest_data {
        Ok(u) => {
            let temp = UserTemplate {
                user: user_data,
                user_guest: u,
            };
            HttpResponse::Ok().body(temp.render_once().unwrap())
        }
        Err(_) => HttpResponse::NotFound().body("User doesn't exist"),
    }
}
