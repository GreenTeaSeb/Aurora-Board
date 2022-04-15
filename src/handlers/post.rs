use crate::handlers::user::{get_by_id, User};
use actix_web::{self, web, HttpMessage, HttpRequest, HttpResponse};
use sailfish::TemplateOnce;
use sqlx::MySqlPool;

#[derive(TemplateOnce)]
#[template(path = "newpost.stpl", escape = false)]
struct NewPostTemplate {
    user: anyhow::Result<User>,
}

pub async fn newpost_pg(req: HttpRequest, pool: web::Data<MySqlPool>) -> HttpResponse {
    let id = req.extensions().get::<u64>().unwrap().to_owned();
    let temp = NewPostTemplate {
        user: get_by_id(&id, pool.get_ref()).await,
    };
    HttpResponse::Ok().body(temp.render_once().unwrap())
}
