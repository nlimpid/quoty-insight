use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, Layer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use entities::*;

use crate::db::Storage;

mod quote_server;
mod db;
mod entities;

#[tokio::main]
async fn main() {
    let filter_layer = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // Set up the JSON formatting layer
    let formatter = fmt::layer()
        .json()  // Output logs in JSON format
        .with_filter(filter_layer);

    // Set up the subscriber to use the JSON formatter
    tracing_subscriber::registry()
        .with(formatter)
        .init();


    let p = Storage::new().await;
    let mut h = crate::quote_server::QuoteServer::new().await;
    let sub_list = p.get_sub_list().await;
    info!("sub_list length is {}, first is {:?}", sub_list.len(), sub_list.first());
    h.sub(sub_list.clone()).await;
    h.start_quote_server(&p).await;
}
