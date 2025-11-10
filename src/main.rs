use tokio::sync::watch;
use tower_http::trace::TraceLayer;
use via_core_ext::{config::Config, state::AppState};

use axum::http::{Request, Response};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use vise_exporter::MetricsExporter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env()?;

    let state = AppState::new(config.clone()).await?;

    let app = state.into_router().layer(
        TraceLayer::new_for_http()
            .make_span_with(|req: &Request<_>| {
                tracing::info_span!(
                    "request",
                    method = %req.method(),
                    uri = %req.uri(),
                )
            })
            .on_request(|_req: &Request<_>, _span: &tracing::Span| {
                tracing::info!("Started processing request");
            })
            .on_response(
                |res: &Response<_>, latency: std::time::Duration, _span: &tracing::Span| {
                    tracing::info!(
                        status = %res.status(),
                        took_ms = latency.as_millis(),
                        "Finished processing request"
                    );
                },
            )
            .on_failure(
                |error: tower_http::classify::ServerErrorsFailureClass,
                 latency: std::time::Duration,
                 _span: &tracing::Span| {
                    tracing::error!(
                        error = %error,
                        took_ms = latency.as_millis(),
                        "Request failed"
                    );
                },
            ),
    );

    let (shutdown_sender, mut shutdown_receiver) = watch::channel(());
    let exporter = MetricsExporter::default().with_graceful_shutdown(async move {
        shutdown_receiver.changed().await.ok();
    });
    tokio::spawn(exporter.start(config.metrics_address.parse().unwrap()));

    let listener = tokio::net::TcpListener::bind(&config.app_address).await?;
    tracing::info!("🚀 Server listening on {}", config.app_address);

    axum::serve(listener, app).await?;

    shutdown_sender.send_replace(());

    Ok(())
}
