use crate::handlers::user::{get_by_id, User};
use actix_identity::Identity;
use actix_web::{get, web, HttpResponse};
use sailfish::TemplateOnce;
use sqlx::MySqlPool;

#[derive(TemplateOnce)]
#[template(path = "home.stpl", escape = false)]
struct HomeTemplate {
    user: anyhow::Result<User>,
}

#[get("/")]
pub async fn home(id: Identity, pool: web::Data<MySqlPool>) -> HttpResponse {
    let id_int: u64 = id
        .identity()
        .unwrap_or_default()
        .parse()
        .unwrap_or_default();
    let user_res = get_by_id(&id_int, pool.get_ref()).await;
    let temp = HomeTemplate { user: user_res };
    HttpResponse::Ok().body(temp.render_once().unwrap())
}
