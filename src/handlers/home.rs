use crate::handlers::user::{get_by_id, User};
use actix_identity::Identity;
use actix_web::{web, HttpResponse};
use sailfish::TemplateOnce;
use sqlx::MySqlPool;

#[derive(TemplateOnce)]
#[template(path = "home.stpl", escape = false)]
struct HomeTemplate {
    user: anyhow::Result<User>,
    top_boards: Vec<BoardEntry>,
}

struct BoardEntry {
    name: String,
    member_count: u64,
    icon: String,
}

async fn get_top(pool: &MySqlPool) -> Result<Vec<BoardEntry>, sqlx::Error> {
    sqlx::query_as!(
        BoardEntry,
        r#"
   SELECT boards.name, cast(count(members.board_id) as UNSIGNED) as member_count, boards.icon  from boards
left join members
on boards.id = members.board_id
group by boards.id
order by member_count desc
limit 10;"#
    )
    .fetch_all(pool)
    .await
}

pub async fn home(id: Identity, pool: web::Data<MySqlPool>) -> HttpResponse {
    let id_int: u64 = id
        .identity()
        .unwrap_or_default()
        .parse()
        .unwrap_or_default();
    let user_res = get_by_id(&id_int, pool.get_ref()).await;
    let top: Vec<BoardEntry> = get_top(pool.get_ref()).await.unwrap_or_default();
    let temp = HomeTemplate {
        user: user_res,
        top_boards: top,
    };
    HttpResponse::Ok().body(temp.render_once().unwrap())
}
