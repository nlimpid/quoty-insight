use std::collections::HashMap;

use anyhow::Context;
use config::Config;
use influxdb3_client::Client;
use influxdb3_client::Precision;
use influxdb3_types::http::QueryFormat;
use serde::Deserialize;

use std::fmt;

#[derive(Debug, Clone, PartialEq)] // Add derives as needed
pub enum FieldValue {
    String(String),
    Float(f64),
    Integer(i64), // Use i64 for InfluxDB integers
    Boolean(bool),
}

// Optional: Implement Display for easy formatting later
impl fmt::Display for FieldValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // IMPORTANT: Add quotes only for String variant
            FieldValue::String(s) => write!(f, "\"{}\"", s.replace('"', "\\\"")), // Escape quotes within the string
            FieldValue::Float(v) => write!(f, "{}", v),
            FieldValue::Integer(v) => write!(f, "{}i", v), // Add 'i' suffix for integers
            FieldValue::Boolean(v) => write!(f, "{}", v),
        }
    }
}

#[derive(Debug, Clone)]
pub struct InfluxDB {
    client: Client,
}

pub struct Point {
    pub name: String,
    pub tags: HashMap<String, String>,
    pub fields: HashMap<String, FieldValue>,
}

#[derive(Clone, Debug, Deserialize)]
struct InfluxDBConfig {
    pub url: String,
    pub token: String,
}

impl InfluxDB {
    pub fn new() -> Self {
        let cfg: InfluxDBConfig = Config::builder()
            .add_source(config::Environment::with_prefix("INFLUXDB"))
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap();
        let client = Client::new(cfg.url, None)
            .unwrap()
            .with_auth_token(cfg.token);
        Self { client }
    }

    pub async fn write_batch(&self, data: Vec<Point>) -> anyhow::Result<()> {
        let input = data
            .iter()
            .map(|p| {
                format!(
                    "{},{} {}",
                    p.name,
                    p.tags
                        .iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect::<Vec<String>>()
                        .join(","),
                    p.fields
                        .iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect::<Vec<String>>()
                        .join(",")
                )
            })
            .collect::<Vec<String>>()
            .join("\n");
        println!("input: {}", input);
        self.write(input).await
    }

    pub async fn write(&self, data: String) -> anyhow::Result<()> {
        self.client
            .api_v3_write_lp("quoty")
            .precision(Precision::Auto)
            // .accept_partial(true)
            .body(data)
            .send()
            .await
            .context("send write_lp request")
    }

    pub async fn query(
        &self,
        query: &str,
        format: Option<QueryFormat>,
    ) -> anyhow::Result<serde_json::Value> {
        let res = self
            .client
            .api_v3_query_sql("quoty", query)
            .format(format.unwrap_or(QueryFormat::Json))
            .send()
            .await
            .context("send query request")?;
        let dd = serde_json::from_slice::<serde_json::Value>(&res).unwrap();
        Ok(dd)
    }
}

mod tests {
    use std::hash::Hash;

    use super::*;

    #[tokio::test]
    async fn test_query() {
        let influxdb = InfluxDB::new();
        let res = influxdb
            .query(
                "select symbol, timestamp, price from trade where symbol = 'AAPL.US' ORDER BY timestamp DESC",
                None,
            )
            .await
            .unwrap();
        let data = res.as_array().unwrap();
        let dd = data
            .iter()
            .map(|d| {
                println!("d: {:?}", d);
            })
            .collect::<Vec<_>>();
    }

    #[tokio::test]
    async fn test_write_batch() {
        let influxdb = InfluxDB::new();
        let points = vec![
            Point {
                name: "demo".to_string(),
                tags: HashMap::from([("tag1".to_string(), "tag_value1".to_string())]),
                fields: HashMap::from([(
                    "field1".to_string(),
                    FieldValue::String("field_value1".to_string()),
                )]),
            },
            Point {
                name: "demo".to_string(),
                tags: HashMap::from([("tag1".to_string(), "tag_value".to_string())]),
                fields: HashMap::from([(
                    "field1".to_string(),
                    FieldValue::String("field_value".to_string()),
                )]),
            },
        ];
        let res = influxdb.write_batch(points).await.unwrap();
        println!("res: {:?}", res);

        let res = influxdb.query("select * from demo", None).await.unwrap();
        println!("res: {:?}", res);
    }
}
