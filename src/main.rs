use actix_files as fs;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{
    middleware::{self, ErrorHandlerResponse}, services,
    web::{self, Data},
    App, HttpRequest, HttpServer, Responder, dev,
};
use anyhow::Result;
use dotenv::dotenv;
use sqlx::MySqlPool;
use std::env;
mod handlers;


#[actix_web::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE is not set");
    let host = env::var("HOST").expect("HOST ip is not set");
    let port = env::var("PORT")
        .expect("PORT is not set")
        .parse::<u16>()
        .expect("Port must be a number");
    let db_pool = MySqlPool::connect(&database_url).await?;
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(db_pool.clone()))
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&[0; 32]).name("id").secure(false),
            ))
            .wrap(middleware::Logger::default())
            .service(services![
                handlers::login::signup,
                handlers::login::login,
                handlers::login::logout,
                handlers::login::login_pg,
                handlers::login::signup_pg,
                handlers::user::get,
                handlers::home::home,
               
            ]).service(web::scope("/b").service(
		services![
		    handlers::board::board_pg,
		    handlers::board::join_board,
		    handlers::board::leave_board,
		]
	    )).service(web::scope("/new").service(
		services![
		    handlers::board::newboard_pg,
		    handlers::board::newboard,
		]
	    ))
            .service(fs::Files::new("/css", "./css/").show_files_listing())
            .service(fs::Files::new("/data", "./data/").show_files_listing())
            .service(fs::Files::new("/", "./static/").show_files_listing())
    })
    .bind((host, port))?
    .run()
    .await?;
    Ok(())
}
