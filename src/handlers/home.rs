use actix_identity::Identity;
use actix_web::{get, HttpResponse, web};
use sailfish::TemplateOnce;
use sqlx::MySqlPool;
use crate::handlers::login::{User, get_by_id};
#[derive(TemplateOnce)]
#[template(path = "home.stpl", escape = false)]
struct homeTemplate {
    user: anyhow::Result<User>,
    posts: Vec<String>,
}


#[get("/")]
pub async fn home(id: Identity, pool: web::Data<MySqlPool>) -> HttpResponse {
    let id_int: u64 = id.identity().unwrap_or_default().parse().unwrap_or_default();
    let user_res = get_by_id(&id_int, pool.get_ref()).await;
    let temp = homeTemplate{
	user: user_res,
	posts: Vec::<String>::default()
    };
    HttpResponse::Ok().body(temp.render_once().unwrap())
}

