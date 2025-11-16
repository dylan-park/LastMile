use surrealdb::{
    Surreal,
    engine::local::{Db, Mem},
};

/// Create a fresh in-memory test database with schema
pub async fn setup_test_db() -> Surreal<Db> {
    let db = Surreal::new::<Mem>(())
        .await
        .expect("Failed to create in-memory DB");
    db.use_ns("test")
        .use_db("test")
        .await
        .expect("Failed to use namespace/database");

    // Setup schema using the same function as production
    lastmile::db::setup_database(&db).await;

    db
}
