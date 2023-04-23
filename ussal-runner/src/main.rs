use axum::routing::get;
use axum::Router;
use clap::Parser;
use cli::{Args, Mode};
use config::OrchestratorConfig;
use std::net::{Ipv6Addr, SocketAddr};

mod cli;
mod config;
mod install;
mod job_handler;
mod letsencrypt;
mod runner;
mod status_page;
mod system;

#[tokio::main]
async fn main() {
    let (non_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout());
    tracing_subscriber::fmt().with_writer(non_blocking).init();

    let args = Args::parse();
    match args.mode {
        Mode::Runner => todo!("Implement runner"),
        Mode::Orchestrator => orchestrator(args).await,
        Mode::OrchestratorAndRunner => orchestrator(args).await,
        Mode::DestructivelyInstallRunner => install::install_runner(args),
    }
}

async fn orchestrator(args: Args) {
    let _config = OrchestratorConfig::load();
    let acceptor = letsencrypt::acme(&args).await;

    let app = Router::new()
        .route("/", get(status_page::show_status))
        //.route("/request_job", get(job_handler::request_job)) // turn connections into websocket, store websocket in state
        .route("/run_job", get(job_handler::run_job));

    let addr = SocketAddr::from((Ipv6Addr::UNSPECIFIED, args.port));

    tracing::info!("Starting HTTPS on port: {}", args.port);
    axum_server::bind(addr)
        .acceptor(acceptor)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
