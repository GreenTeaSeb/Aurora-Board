use crate::handlers::user::{self, User};
use actix_files as fs;
use actix_identity::Identity;
use actix_web::{self, get, post, web, HttpRequest, HttpResponse, Responder};
use anyhow::{anyhow, Result};
use sailfish::TemplateOnce;
use serde::{Deserialize, Serialize};
use sqlx::{types::chrono, FromRow, MySqlPool};


