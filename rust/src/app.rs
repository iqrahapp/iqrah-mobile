use crate::repository::LearningService;
use crate::sqlite_repo::SqliteRepository;
use anyhow::Result;
use once_cell::sync::OnceCell;
use std::path::PathBuf;
use std::sync::Arc;

pub struct App {
    pub service: LearningService,
}

static APP: OnceCell<App> = OnceCell::new();

pub fn init_app(path: Option<PathBuf>) -> Result<()> {
    if APP.get().is_some() {
        return Ok(()); // Already initialized
    } else {
        let repo = Arc::new(SqliteRepository::new(path)?);
        repo.seed()?; // Separate seeding step

        let service = LearningService::new(repo);
        APP.set(App { service })
            .map_err(|_| anyhow::anyhow!("App already initialized"))?;
        Ok(())
    }
}

pub fn app() -> &'static App {
    APP.get().expect("App not initialized")
}
