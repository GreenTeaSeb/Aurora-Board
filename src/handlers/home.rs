use std::default;

use actix_identity::Identity;
use actix_web::{get, HttpResponse};
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "home.stpl", escape = false)]
struct homeTemplate {
    user: Option<String>,
    posts: Vec<String>,
}


#[get("/")]
pub async fn home(id: Identity) -> HttpResponse {

    let temp = homeTemplate{
	user: id.identity(),
	..Default::default()
    };
    HttpResponse::Ok().body(temp.render_once().unwrap())
}

