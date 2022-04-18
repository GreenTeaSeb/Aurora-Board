use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse};
use anyhow::Result;
use sailfish::TemplateOnce;
use sqlx::MySqlPool;

#[derive(Debug)]
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
    user: Result<User>,
    user_guest: User,
}

pub async fn get_by_id(id: &u32, pool: &MySqlPool) -> Result<User> {
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

pub async fn get(id: Identity, req: HttpRequest, pool: web::Data<MySqlPool>) -> HttpResponse {
    let id_guest = req.match_info().get("id").ok_or("no such id");
    let id_guest_int: u32 = id_guest.unwrap_or_default().parse().unwrap_or_default();
    let user_guest_data = get_by_id(&id_guest_int, pool.get_ref()).await;
    let id_int: u32 = id
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

pub async fn check_if_joined_board(id: u32, board: &str, pool: &MySqlPool) -> Result<bool> {
    Ok(sqlx::query!(
        r#"
SELECT count(*) AS is_in FROM members
WHERE board_id = (SELECT id FROM boards WHERE name = ?) AND user_id = ?
"#,
        board,
        id
    )
    .fetch_one(pool)
    .await?
    .is_in
        != 0)
}

pub struct UserBoards {
    pub name: String,
    pub icon: String,
}
pub async fn get_user_boards(id: u32, pool: &MySqlPool) -> Vec<UserBoards> {
    sqlx::query_as!(
        UserBoards,
        r#"
select  boards.name, boards.icon from boards
inner join members on
members.board_id = boards.id and user_id = ?
"#,
        id
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default()
}
