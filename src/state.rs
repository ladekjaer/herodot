use crate::repository::Repository;

#[derive(Clone)]
pub(crate) struct AppState {
    pub repository: Repository,
}

impl AppState {
    pub fn new(db_pool: sqlx::PgPool) -> Self {
        Self {
            repository: Repository::new(db_pool),
        }
    }
}
