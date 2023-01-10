use crate::rocket::futures::TryFutureExt;
use rocket_db_pools::sqlx::{self, Row};
use rocket_db_pools::Connection;

use crate::Db;
use rocket::response::status::Created;
use rocket::serde::{json::Json, Deserialize, Serialize};

type Result<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;
