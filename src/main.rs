use actix_files as fs;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::web::{self, Data};
use actix_web::{App, HttpServer};
use anyhow::Result;
use dotenv::dotenv;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
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
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(db_pool.clone()))
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&[0; 32]).name("id").secure(false),
            ))
            .route("/", web::get().to(handlers::home::home))
            .route("/login", web::get().to(handlers::login::login_pg))
            .route("/signup", web::get().to(handlers::login::signup_pg))
            .route("/logout", web::get().to(handlers::login::logout))
            .route("/signup", web::post().to(handlers::login::signup))
            .route("/login", web::post().to(handlers::login::login))
            .service(web::scope("/users").route("{id}", web::get().to(handlers::user::get)))
            .service(
                web::scope("/boards")
                    .route("{name}", web::get().to(handlers::board::board_pg))
                    .service(
                        web::scope("{name}")
                            .wrap(handlers::middleware::LoginAuth)
                            .route("join", web::post().to(handlers::board::join_board))
                            .route("leave", web::post().to(handlers::board::leave_board)),
                    )
                    .service(
                        web::scope("")
                            .wrap(handlers::middleware::LoginAuth)
                            .route("", web::post().to(handlers::board::newboard))
                            .route("", web::get().to(handlers::board::newboard_pg)),
                    ),
            )
            .service(fs::Files::new("/data", "./data/").show_files_listing())
            .service(fs::Files::new("", "./static/").show_files_listing())
    })
    .bind_openssl((host, port), builder)?
    // .bind((host,port))?
    .run()
    .await?;
    Ok(())
}
