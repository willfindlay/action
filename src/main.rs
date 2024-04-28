#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod hooks;
mod template;
mod tracker;

use anyhow::Result;
use axum::{routing::get, Router};
use hooks::listen;
use std::{env, sync::Arc, thread};
use template::{APMData, APMTemplate};
use tokio::sync::{mpsc::unbounded_channel, Mutex};
use tracker::Tracker;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenv::dotenv();
    tracing_subscriber::fmt::init();
    let port = env::var("ACTION_PORT").unwrap_or_else(|_| "3333".into());
    tracing::info!(port = port, "Configured port");

    let tracker = Arc::new(Tracker::default());

    let (schan, rchan) = unbounded_channel();
    let _listener = thread::spawn(move || {
        listen(move || {
            schan
                .send(())
                .unwrap_or_else(|e| tracing::error!(err = ?e, "Could not send event"))
        })
        .expect("could not listen");
    });

    {
        let tracker = tracker.clone();
        tracker.track(Arc::new(Mutex::new(rchan))).await?;
    }

    let app = Router::new()
        .route(
            "/",
            get(|| async move {
                APMTemplate {
                    css: include_str!("../style.css"),
                }
            }),
        )
        .route(
            "/apm",
            get(|| async move {
                axum::Json(APMData {
                    apm: tracker.apm().await,
                })
            }),
        );
    tracing::info!("Starting webserver");
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
