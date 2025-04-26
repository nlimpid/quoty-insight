use std::collections::HashMap;

use crate::quote_server::QuoteServer;
use egress::influxdb::FieldValue;
use egress::influxdb::Point;
use sink::{Sink, YamlSink};
use tokio::select;
use tokio::signal;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter, Layer};

mod egress;
mod quote_server;
mod sink;

use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    config: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    println!("config: {}", args.config);
    let sink = YamlSink::new(&args.config);
    let sub_list = sink.get_sub_list();

    let filter_layer = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // Set up the JSON formatting layer
    let formatter = fmt::layer()
        .json() // Output logs in JSON format
        .with_filter(filter_layer);

    // Set up the subscriber to use the JSON formatter
    tracing_subscriber::registry().with(formatter).init();

    let (mut h, mut dispatchers) = QuoteServer::new().await.unwrap();
    h.sub(sub_list).await;
    h.start_dispatcher().unwrap();

    write_to_influxdb(dispatchers);

    let sig = signal::ctrl_c();
    select! {
        _ = sig => {
            println!("Ctrl-C received, exiting...");
        }
    }
}

fn write_to_influxdb(mut dispatchers: quote_server::QuoteDispatchers) {
    let influxdb = egress::influxdb::InfluxDB::new();
    let influxdb_clone = influxdb.clone();

    tokio::spawn(async move {
        while let Some(trade) = dispatchers.trade_rx.recv().await {
            println!("trade: symbol {}, price {:#?}", trade.symbol, trade.data);
            let points = trade
                .data
                .trades
                .iter()
                .map(|t| Point {
                    name: "trade".to_string(),
                    tags: HashMap::from([("symbol".to_string(), trade.symbol.clone())]),
                    fields: HashMap::from([
                        ("price".to_string(), FieldValue::String(t.price.to_string())),
                        ("volume".to_string(), FieldValue::Integer(t.volume)),
                        (
                            "timestamp".to_string(),
                            FieldValue::String(t.timestamp.to_string()),
                        ),
                        (
                            "trade_type".to_string(),
                            FieldValue::String(t.trade_type.to_string()),
                        ),
                        (
                            "trade_direction".to_string(),
                            FieldValue::Integer(t.direction as i64),
                        ),
                        (
                            "trade_session".to_string(),
                            FieldValue::Integer(t.trade_session as i64),
                        ),
                    ]),
                })
                .collect::<Vec<Point>>();
            let res = influxdb.write_batch(points).await;
            if res.is_err() {
                println!("write_batch error: {:?}", res);
            }
        }
    });

    tokio::spawn(async move {
        while let Some(depth) = dispatchers.depth_rx.recv().await {
            // ignore
            // println!("depth: symbol {}, price {:#?}", depth.symbol, depth.data);
        }
    });

    tokio::spawn(async move {
        while let Some(quote) = dispatchers.quote_rx.recv().await {
            // ignore
            // println!("quote: symbol {}, price {:#?}", quote.symbol, quote.data);

            let point = Point {
                name: "quote".to_string(),
                tags: HashMap::from([("symbol".to_string(), quote.symbol.clone())]),
                fields: HashMap::from([
                    (
                        "last_done".to_string(),
                        FieldValue::String(quote.data.last_done.to_string()),
                    ),
                    (
                        "open".to_string(),
                        FieldValue::String(quote.data.open.to_string()),
                    ),
                    (
                        "high".to_string(),
                        FieldValue::String(quote.data.high.to_string()),
                    ),
                    (
                        "low".to_string(),
                        FieldValue::String(quote.data.low.to_string()),
                    ),
                ]),
            };
            let res = influxdb_clone.write_batch(vec![point]).await;
            if res.is_err() {
                println!("write_batch error: {:?}", res);
            }
        }
    });
}
