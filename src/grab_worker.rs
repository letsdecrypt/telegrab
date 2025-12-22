use std::time::Duration;

use sqlx::{postgres::PgPoolOptions, prelude::FromRow, Executor, PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{configuration::Settings,Result};

pub async fn run_worker_until_stopped(configuration: Settings) -> Result<()>{
    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());
    worker_loop(connection_pool).await
}

async fn worker_loop(pool: PgPool) -> Result<()> {
    loop {
        match try_execute_task(&pool).await {
            Ok(ExecutionOutcome::EmptyQueue) => {
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
            Err(_) => {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            Ok(ExecutionOutcome::TaskCompleted) => {}
        }
    }
}
pub enum ExecutionOutcome {
    TaskCompleted,
    EmptyQueue,
}

pub async fn try_execute_task(pool: &PgPool) -> Result<ExecutionOutcome> {
    Ok(ExecutionOutcome::EmptyQueue)
}