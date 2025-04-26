use std::collections::HashMap;
use std::sync::Arc;

use longport::quote::SubFlags;
use longport::quote::{PushDepth, PushEvent, PushEventDetail, PushQuote, PushTrades};
use longport::{Config, Decimal, Market, QuoteContext};
// use crate::channels;
use anyhow::Result;
use tokio::sync::mpsc;

pub(crate) struct QuoteServer {
    quote_ctx: QuoteContext,
    // Egress is removed - downstream consumers handle egress

    // Keep the channel fed by QuoteContext's receiver
    price_rx: Option<mpsc::Receiver<PushEvent>>, // Make Option to take ownership in dispatcher

    // Senders for dispatched types
    quote_tx: mpsc::Sender<Quote>,
    trade_tx: mpsc::Sender<Trade>,
    depth_tx: mpsc::Sender<Depth>,
}

// Struct to hold the receiver ends for consumers
pub struct QuoteDispatchers {
    pub quote_rx: mpsc::Receiver<Quote>,
    pub trade_rx: mpsc::Receiver<Trade>,
    pub depth_rx: mpsc::Receiver<Depth>,
}

#[derive(Debug)]
pub struct Quote {
    pub symbol: String,
    pub data: PushQuote,
}

#[derive(Debug)]
pub struct Trade {
    pub symbol: String,
    pub data: PushTrades,
}

#[derive(Debug)]
pub struct Depth {
    pub symbol: String,
    pub data: PushDepth,
}

impl QuoteServer {
    // New returns the server and the dispatcher struct with receivers
    pub async fn new() -> anyhow::Result<(Self, QuoteDispatchers)> {
        let config = Arc::new(Config::from_env()?);
        let (ctx, mut main_receiver) = QuoteContext::try_new(config).await?;

        // Channel for raw events from QuoteContext receiver -> dispatcher task
        // Increased buffer size for the raw intake channel potentially
        let (internal_event_tx, internal_event_rx) = mpsc::channel::<PushEvent>(500_000);

        // Task to forward from LongPort receiver to our internal channel
        tokio::spawn(async move {
            while let Some(event) = main_receiver.recv().await {
                if internal_event_tx.send(event).await.is_err() {
                    eprintln!("Internal event channel closed. Forwarder stopping.");
                    break; // Exit if the receiver is dropped
                }
            }
        });

        // Create specific channels for dispatching
        // Consider buffer sizes based on expected downstream processing rate
        let (quote_tx, quote_rx) = mpsc::channel::<Quote>(10000);
        let (trade_tx, trade_rx) = mpsc::channel::<Trade>(10000);
        let (depth_tx, depth_rx) = mpsc::channel::<Depth>(10000);

        let server = QuoteServer {
            quote_ctx: ctx,
            price_rx: Some(internal_event_rx), // Store receiver for the dispatcher
            quote_tx,
            trade_tx,
            depth_tx,
        };

        let dispatchers = QuoteDispatchers {
            quote_rx,
            trade_rx,
            depth_rx,
        };

        Ok((server, dispatchers))
    }

    // Spawns the dispatcher task that consumes price_rx
    pub fn start_dispatcher(&mut self) -> Result<(), &'static str> {
        if self.price_rx.is_none() {
            return Err("Dispatcher already started or receiver taken.");
        }
        let mut receiver = self.price_rx.take().unwrap(); // Take ownership of the receiver

        // Clone senders for the spawned task
        let quote_sender = self.quote_tx.clone();
        let trade_sender = self.trade_tx.clone();
        let depth_sender = self.depth_tx.clone();

        tokio::spawn(async move {
            println!("Starting event dispatcher...");
            while let Some(event) = receiver.recv().await {
                let symbol = event.symbol.clone(); // Clone symbol for use in dispatched data

                match event.detail {
                    PushEventDetail::Quote(data) => {
                        if quote_sender.send(Quote { symbol, data }).await.is_err() {
                            eprintln!("Quote channel closed. Dispatcher stopping.");
                            break; // Stop if quote consumer is gone
                        }
                    }
                    PushEventDetail::Trade(data) => {
                        if trade_sender.send(Trade { symbol, data }).await.is_err() {
                            eprintln!("Trade channel closed. Dispatcher stopping.");
                            break; // Stop if trade consumer is gone
                        }
                    }
                    PushEventDetail::Depth(data) => {
                        if depth_sender.send(Depth { symbol, data }).await.is_err() {
                            eprintln!("Depth channel closed. Dispatcher stopping.");
                            break; // Stop if depth consumer is gone
                        }
                    }
                    _ => {
                        // Optional: Log unhandled event types
                        // println!("Ignoring event type: {:?}", event.detail);
                    }
                }
            }
            println!("Event dispatcher task finished.");
        });
        Ok(()) // Indicate dispatcher task was spawned
    }

    pub async fn quote_basic(&self, ticker_region_list: Vec<String>) -> HashMap<String, Decimal> {
        let resp = self.quote_ctx.quote(ticker_region_list).await.unwrap();
        resp.iter()
            .map(|q| (q.symbol.clone(), q.last_done.clone()))
            .collect()
    }

    pub async fn sub(&mut self, ticker_region_list: Vec<String>) {
        self.quote_ctx
            .subscribe(ticker_region_list.clone(), SubFlags::all(), true)
            .await
            .unwrap();
        println!("sub finished");
    }
}
