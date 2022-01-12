use crate::handlers::user::{get_by_id, User};
use actix_files as fs;
use actix_identity::Identity;
use actix_web::{self, get, post, web, HttpResponse, Responder};
use anyhow::{anyhow, Result};
use argonautica::{Hasher, Verifier};
use serde::{Deserialize, Serialize};
use sqlx::{MySqlPool};
use std::env;

#[derive(Serialize)]
pub struct Error {
    text: String,
}

#[derive(Deserialize)]
pub struct SignupData {
    username: String,
    email: String,
    password: String,
}

#[derive(Deserialize)]
pub struct LoginData {
    email: String,
    password: String,
}

async fn new_user(form: SignupData, pool: &MySqlPool) -> Result<User> {
    let users = sqlx::query_as!(
        User,
        "
SELECT * FROM users
WHERE email = ?
    ",
        form.email
    )
    .fetch_all(pool)
    .await?;

    if users.is_empty() {
        let key = env::var("KEY").expect("SECRET KEY not set");
        let pass = match Hasher::default()
            .with_password(form.password)
            .with_secret_key(key)
            .configure_hash_len(32)
            .hash()
        {
            Ok(pass) => pass,
            Err(e) => return Err(anyhow!("Error with hash {}", e)),
        };

        let new = sqlx::query!(
            r#"
	    INSERT INTO users (username, email, password)
	    VALUES (? , ? , ?)
	"#,
            form.username,
            form.email,
            pass
        )
        .execute(pool)
        .await?;

        let user = get_by_id(&new.last_insert_id(), pool).await?;
        Ok(user)
    } else {
        return Err(anyhow!("email already in use"));
    }
}

async fn login_user(form: LoginData, pool: &MySqlPool) -> Result<User> {
    let user = match sqlx::query_as!(
        User,
        r#"
	SELECT * FROM users
	WHERE email = ? 
    "#,
        form.email
    )
    .fetch_one(pool)
    .await
    {
        Ok(u) => u,
        Err(_) => return Err(anyhow!("User doesn't exist")),
    };

    let key = env::var("KEY").expect("SECRET KEY not set");
    let hash = String::from_utf8_lossy(&user.password);
    let pass = match Verifier::default()
        .with_hash(hash)
        .with_password(&form.password)
        .with_secret_key(key)
        .verify()
    {
        Ok(pass) => pass,
        Err(e) => return Err(anyhow!("Error with hash {}", e)),
    };
    if pass {
        Ok(user)
    } else {
        return Err(anyhow!("Wrong password"));
    }
}

#[post("/signup")]
pub async fn signup(
    id: Identity,
    form: web::Form<SignupData>,
    db_pool: web::Data<MySqlPool>,
) -> impl Responder {
    match new_user(form.into_inner(), db_pool.as_ref()).await {
        Ok(t) => {
            id.remember(t.id.to_string());
            HttpResponse::Found()
                .append_header(("location", "/"))
                .finish()
        }
        Err(e) => HttpResponse::UnprocessableEntity().body(e.to_string()),
    }
}

#[post("/login")]
pub async fn login(
    id: Identity,
    form: web::Form<LoginData>,
    db_pool: web::Data<MySqlPool>,
) -> impl Responder {
    match login_user(form.into_inner(), db_pool.as_ref()).await {
        Ok(t) => {
            id.remember(t.id.to_string());
            HttpResponse::Found()
                .append_header(("location", "/"))
                .finish()
        }
        Err(e) => HttpResponse::UnprocessableEntity().body(e.to_string()),
    }
}

#[get("/logout")]
pub async fn logout(id: Identity) -> impl Responder {
    id.forget();
    HttpResponse::Found()
        .append_header(("location", "/"))
        .finish()
}

#[get("login")]
pub async fn login_pg() -> std::io::Result<fs::NamedFile> {
    Ok(fs::NamedFile::open("./static/login.html")?)
}
#[get("signup")]
pub async fn signup_pg() -> std::io::Result<fs::NamedFile> {
    Ok(fs::NamedFile::open("./static/signup.html")?)
}
