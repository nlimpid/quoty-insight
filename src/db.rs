use std::env;
use std::error::Error;
use std::time::Duration;

use dotenv::dotenv;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, EntityTrait, InsertResult};
use sqlx::{Connection, ConnectOptions};
use migration::{Migrator, MigratorTrait};
use url::Url;
use crate::entities::quote_price::Entity as QuotePrice;
use crate::entities::quote_sub::Entity as QuoteSub;
use crate::entities::quote_trade::Entity as QuoteTrade;
use crate::entities::{quote_price, quote_trade};

pub struct Storage {
    db: sea_orm::DatabaseConnection,
}

impl Storage {
    pub async fn new() -> Self {
        dotenv().ok().unwrap();

        let database_url = env::var("DATABASE_URL").unwrap();

        println!("database_url is {}", &database_url);
        let url = Url::parse(&database_url).unwrap();

        let sqlx_options = sqlx::mysql::MySqlConnectOptions::from_url(&url).unwrap();
        // let sqlx_conn = sqlx::MySqlConnection::connect(&database_url).await?;
        let sqlx_pool = sqlx::MySqlPool::connect_with(sqlx_options).await.unwrap();
        let db = sea_orm::SqlxMySqlConnector::from_sqlx_mysql_pool(sqlx_pool);
        // // let mut opt = sea_orm::ConnectOptions::new(&database_url);
        // opt.max_connections(100)
        //     .min_connections(5)
        //     .connect_timeout(Duration::from_secs(8))
        //     .acquire_timeout(Duration::from_secs(8))
        //     .idle_timeout(Duration::from_secs(8))
        //     .max_lifetime(Duration::from_secs(8));
        //     // .sqlx_logging(true);
        //
        // let db = sea_orm::Database::connect(opt).await.unwrap();
        Migrator::up(&db, None).await.unwrap();

        Storage { db }
    }

    pub async fn batch_insert_price(&self, prices: Vec<quote_price::ActiveModel>) {
        let res: InsertResult<quote_price::ActiveModel> = QuotePrice::insert_many(prices)
            .exec(&self.db)
            .await
            .unwrap();
        println!("insert result is {}", res.last_insert_id);
    }

    pub async fn batch_insert_trade(&self, items: Vec<quote_trade::ActiveModel>) {
        let res: InsertResult<quote_trade::ActiveModel> =
            QuoteTrade::insert_many(items).exec(&self.db).await.unwrap();
        println!("insert result is {}", res.last_insert_id);
    }

    pub async fn get_sub_list(&self) -> Vec<String> {
        let res = QuoteSub::find().all(&self.db).await.unwrap();
        res.iter().map(|i| i.symbol.clone()).collect()
    }
}

mod test {
    use dotenv::dotenv;

    use super::*;

    #[tokio::test]
    async fn test_basic() {
        dotenv().ok().unwrap();
        let p = Storage::new().await;
        let db = &p.db;

        let mut fruit: quote_price::ActiveModel = Default::default();
        assert!(!fruit.is_changed());

        let aap1 = quote_price::ActiveModel {
            symbol: Set(("AAPL1".to_owned())),
            ..Default::default() // all other attributes are `NotSet`
        };

        let pear: quote_price::Model = aap1.insert(db).await.unwrap();
        println!("aap1 is {:?}", pear);

        let items = QuotePrice::find().all(db).await.unwrap();
        println!(" data is {}", items.len());
    }

    #[tokio::test]
    async fn test_sublist() {
        dotenv().ok().unwrap();
        let p = Storage::new().await;
        let db = &p.db;

        let got = p.get_sub_list().await;
        println!("got is {}", got.len())
    }
}
