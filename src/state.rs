use sqlx::{MySql, Pool};

pub struct AppState {
    pub db: Pool<MySql>,
}
