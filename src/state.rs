use crate::seeding::seed_demo_data;

use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use surrealdb::{
    Surreal,
    engine::local::{Db, Mem},
};

#[derive(Clone)]
pub enum DbProvider {
    Single(SingleDbProvider),
    Demo(DemoDbProvider),
}

impl DbProvider {
    pub async fn get_db(&self, session_id: Option<&str>) -> surrealdb::Result<Surreal<Db>> {
        match self {
            DbProvider::Single(p) => p.get_db(session_id).await,
            DbProvider::Demo(p) => p.get_db(session_id).await,
        }
    }
}

#[derive(Clone)]
pub struct SingleDbProvider {
    pub db: Surreal<Db>,
}

impl SingleDbProvider {
    async fn get_db(&self, _session_id: Option<&str>) -> surrealdb::Result<Surreal<Db>> {
        Ok(self.db.clone())
    }
}

#[derive(Clone)]
pub struct DemoDbProvider {
    // Map: SessionID -> (DB Instance, Last Access Time)
    sessions: Arc<DashMap<String, (Surreal<Db>, Instant)>>,
}

impl DemoDbProvider {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
        }
    }

    pub fn cleanup_old_sessions(&self) {
        let now = Instant::now();
        // Remove sessions older than 1 hour
        self.sessions.retain(|_, (_, last_access)| {
            now.duration_since(*last_access) < Duration::from_secs(3600)
        });
    }

    async fn get_db(&self, session_id: Option<&str>) -> surrealdb::Result<Surreal<Db>> {
        let id = session_id.unwrap_or("default");

        if let Some(mut entry) = self.sessions.get_mut(id) {
            entry.1 = Instant::now();
            return Ok(entry.0.clone());
        }

        // Create new in-memory DB
        let db = Surreal::new::<Mem>(()).await?;
        db.use_ns("lastmile").use_db("demo").await?;

        // Seed data
        if let Err(e) = seed_demo_data(&db).await {
            tracing::error!("Failed to seed demo data for session {}: {:?}", id, e);
            return Err(e);
        }

        self.sessions
            .insert(id.to_string(), (db.clone(), Instant::now()));
        Ok(db)
    }
}

pub struct AppState {
    pub db_provider: Arc<DbProvider>,
    pub is_demo_mode: bool,
}
