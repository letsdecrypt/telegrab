use std::fmt::{self, Display};

use telegrab::state::AppState;
use telegrab::{
    configuration::get_configuration, startup::run_app_until_stopped, telemetry::init, worker::start_background_workers,
    Result,
};
use tokio::task::JoinError;

#[tokio::main]
async fn main() -> Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration.");
    init(&configuration.logger);
    let app_state = AppState::build(&configuration).await;
    let application_task = tokio::spawn(run_app_until_stopped(
        app_state.clone(),
        configuration.clone(),
    ));
    start_background_workers(app_state.clone(), configuration.clone()).await;
    tokio::select! {
        o = application_task => report_exit("API server", o),
    };
    Ok(())
}

fn report_exit(
    task_name: &str,
    outcome: std::result::Result<std::result::Result<(), impl fmt::Debug + Display>, JoinError>,
) {
    match outcome {
        Ok(Ok(())) => {
            tracing::info!("{} has exited", task_name)
        }
        Ok(Err(e)) => {
            tracing::error!("{} failed with error: {}", task_name, e);
        }
        Err(e) => {
            tracing::error!("{} task failed to complete with error{}", task_name, e);
        }
    }
}
