use std::collections::HashMap;
use std::sync::Arc;

use dotenv::dotenv;
use longbridge::{Config, Decimal, QuoteContext};
use longbridge::quote::{PushEvent, PushQuote, SubFlags};
use longbridge::quote::PushEventDetail::Quote;
use tokio::sync::mpsc::UnboundedReceiver;
use tracing::debug;


struct QuoteServer {
    quote_ctx: QuoteContext,
    receiver: UnboundedReceiver<PushEvent>,
}

impl QuoteServer {
    pub async fn new() -> Self {
        let config = Arc::new(Config::from_env().unwrap());
        // Create a context for quote APIs
        let (ctx, receiver) = QuoteContext::try_new(config.clone()).await.unwrap();
        QuoteServer {
            quote_ctx: ctx,
            receiver,
        }
    }

    pub async fn quote_basic(&self, ticker_region_list: Vec<String>) -> HashMap<String, Decimal> {
        let resp = self.quote_ctx.quote(ticker_region_list).await.unwrap();
        resp.iter().map(|q| (q.symbol.clone(), q.last_done.clone())).collect()
    }

    pub async fn sub(&mut self, ticker_region_list: [&str; 4]) {
        self.quote_ctx.subscribe(ticker_region_list.clone(), SubFlags::all(), true)
            .await
            .unwrap();
        println!("sub finsih");
        // Subscribe
        while let Some(event) = self.receiver.recv().await {
            let symbol = event.symbol.clone();
            match event.detail {
                Quote(price) => {
                    debug!("price is {}", price.last_done);
                }
                _ => {}
            }
        }
    }
}


mod test {
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
        h.sub(["700.HK", "AAPL.US", "TSLA.US", "NFLX.US"]).await
    }
}

// pub async fn start_quote_server(db_pool: &DatabaseConnection) {
//     let (tx, rx) = mpsc::channel(100);  // 创建 channel
//
//     // 模拟接收数据和发送到channel
//     tokio::spawn(async move {
//         loop {
//             let data = receive_data().await;
//             tx.send(data).await.unwrap();
//         }
//     });
//
//     // 模拟数据处理任务
//     tokio::spawn(async move {
//         while let Some(data) = rx.recv().await {
//             crate::db::insert_data(db_pool, &data).await.unwrap();
//         }
//     });
// }
//
// async fn receive_data() -> YourDataType {
//     // 模拟数据接收逻辑
// }