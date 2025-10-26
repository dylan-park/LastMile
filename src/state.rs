use surrealdb::Surreal;
use surrealdb::engine::local::Db;

pub struct AppState {
    pub db: Surreal<Db>,
}
