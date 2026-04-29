use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::path::Path;
use auditor_core::error::CcaError;

pub type DbPool = Pool<SqliteConnectionManager>;

pub fn create_pool(db_path: impl AsRef<Path>) -> auditor_core::error::Result<DbPool> {
    let manager = SqliteConnectionManager::file(db_path);
    let pool = Pool::new(manager).map_err(|e| CcaError::Database(e.to_string()))?;

    // Run init migration
    let conn = pool.get().map_err(|e| CcaError::Database(e.to_string()))?;
    let init_sql = crate::migrations::get_init_sql();
    conn.execute_batch(init_sql)
        .map_err(|e| CcaError::Database(e.to_string()))?;

    Ok(pool)
}
