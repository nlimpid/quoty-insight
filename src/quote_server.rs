use std::collections::HashMap;
use std::sync::Arc;

use dotenv::dotenv;
use longbridge::{Config, Decimal, QuoteContext};
use longbridge::quote::{PushEvent, PushEventDetail, PushQuote, PushTrades, SubFlags};
use sea_orm::ActiveValue::Set;
// use crate::channels;
use tokio::sync::mpsc;

pub(crate) struct QuoteServer {
    quote_ctx: QuoteContext,

    price_tx: mpsc::Sender<PushEvent>,
    price_rx: mpsc::Receiver<PushEvent>,
}

impl QuoteServer {
    pub async fn new() -> Self {
        let config = Arc::new(Config::from_env().unwrap());
        // Create a context for quote APIs
        let (ctx, mut receiver) = QuoteContext::try_new(config.clone()).await.unwrap();
        let (tx, rx) = mpsc::channel::<PushEvent>(1000000);  // 创建 channel
        let price_tx = tx.clone();

        // Auto send all event
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                if let Err(e) = price_tx.send(event).await {
                    println!("Failed to send event: {:?}", e);
                }
            }
        });

        QuoteServer {
            quote_ctx: ctx,
            // receiver,

            price_tx: tx,
            price_rx: rx,
        }
    }

    pub async fn quote_basic(&self, ticker_region_list: Vec<String>) -> HashMap<String, Decimal> {
        let resp = self.quote_ctx.quote(ticker_region_list).await.unwrap();
        resp.iter().map(|q| (q.symbol.clone(), q.last_done.clone())).collect()
    }


    pub async fn sub(&mut self, ticker_region_list: Vec<String>) {
        self.quote_ctx.subscribe(ticker_region_list.clone(), SubFlags::all(), true)
            .await
            .unwrap();
        println!("sub finished");
    }


    pub async fn start_quote_server(&mut self, db_pool: &crate::db::Storage) {
        // TODO: maybe refine me to different Vec[T]?
        let mut prices = Vec::new();
        let mut trades = Vec::new();
        println!("start quote server");
        // 模拟数据处理任务
        while let Some(event) = self.price_rx.recv().await {
            println!("event symbol {}", event.symbol.clone());
            match event.detail {
                PushEventDetail::Quote(price) => {
                    prices.push(convert_to_storage(event.symbol, &price));
                }
                PushEventDetail::Trade(trade) => {
                    let mut vals = convert_trade_to_storage(event.symbol, &trade);
                    trades.append(&mut vals);
                }
                _ => {}
            }
            if prices.len() >= 10 {
                println!("batch insert db, first is {:?}", prices.first().clone().unwrap().symbol);
                db_pool.batch_insert_price(prices.clone()).await;
                prices.clear();  // 清空数组以便新一轮收集
            }

            if trades.len() >= 10 {
                println!("batch insert db, first is {:?}", trades.first().clone().unwrap().symbol);
                db_pool.batch_insert_trade(trades.clone()).await;
                trades.clear();  // 清空数组以便新一轮收集
            }
        }
        println!("start quote server done");
    }
}

fn convert_to_storage(symbol: String, price: &PushQuote) -> crate::entities::quote_price::ActiveModel {
    let model = crate::entities::quote_price::ActiveModel {
        symbol: Set(symbol.clone()),
        last_done: Set(Some(price.last_done)),
        open: Set(Some(price.open)),
        high: Set(Some(price.high)),
        low: Set(Some(price.low)),
        timestamp: Set(price.timestamp.unix_timestamp()),
        volume: Set(Some(price.volume)),
        turnover: Set(Some(price.turnover)),
        trade_status: Set(Some(price.trade_status.into())),
        trade_session: Set(Some(price.trade_session.into())),
        ..Default::default()
    };
    return model;
}

fn convert_trade_to_storage(symbol: String, data: &PushTrades) -> Vec<crate::entities::quote_trade::ActiveModel> {
    let models = data.trades.iter().map(|trade| {
        crate::entities::quote_trade::ActiveModel {
            symbol: Set(symbol.clone()),
            price: Set(Some(trade.price)),
            volume: Default::default(),
            timestamp: Set(trade.timestamp.unix_timestamp()),
            trade_type: Set(Some(trade.trade_type.clone())),
            direction: Set(Some(trade.direction as i32)),
            trade_session: Set(Some(i32::from(trade.trade_session))),
            ..Default::default()
        }
    }).collect();

    return models;
}


mod test {
    use crate::db::Storage;

    // use crate::quote_server::QuoteServer;
    use super::*;

    #[tokio::test]
    async fn test_basic_get() {
        dotenv().ok();
        let h = QuoteServer::new().await;
        // Get basic information of securities
        let resp = h.quote_ctx
            .quote(["700.HK", "AAPL.US", "TSLA.US", "NFLX.US"])
            .await.unwrap();
        println!("{:?}", resp);
    }

    #[tokio::test]
    async fn test_sub() {
        dotenv().ok();
        let mut h = QuoteServer::new().await;

        h.sub(["700.HK", "AAPL.US", "TSLA.US", "NFLX.US"]).await;

        println!("start quote server");
        let p = Storage::new().await;
        println!("init storage");
        println!("init storage finished");
        h.start_quote_server(&p).await;
    }
}