use miette::IntoDiagnostic;
use tokio::sync::broadcast;
use tokio::task;
use tracing::Instrument;
use tracing::error;
use tracing::info;
use tracing::info_span;
use tracing::warn;

use crate::client;
use crate::config::Config;
use crate::socket::wrapper::Listener;
use crate::socket::wrapper::SocketAddr;

#[derive(Debug, Clone)]
pub struct Message {
    pub addr: SocketAddr,
    pub payload: String,
    pub from: String,
}

pub async fn setup() -> miette::Result<()> {
    let config = Config::default();
    let listener = Listener::bind(&config.listen).await.expect("bind failed");
    info!(socket = ?config.listen, "listening");

    // Create channel shared among all clients that connect to the server loop.
    let (tx, _) = broadcast::channel::<Message>(10);

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                let tx = tx.clone();
                tokio::spawn(async move {
                    info!(socket = ?addr, "client connected");
                    task::spawn(
                        async move {
                            match client::handle_message(stream, tx, addr).await {
                                Ok(_) => {}
                                Err(err) => error!("client error: {err:?}"),
                            }
                        }
                        .instrument(info_span!("client")),
                    );
                });
            }
            Err(err) => match err.kind() {
                // ignore benign errors
                std::io::ErrorKind::NotConnected => {
                    warn!("listener error {err}");
                }
                _ => {
                    Err(err).into_diagnostic()?;
                }
            },
        }
    }
}
