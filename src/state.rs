use surrealdb::{Surreal, engine::local::Db};

pub struct AppState {
    pub db: Surreal<Db>,
}
