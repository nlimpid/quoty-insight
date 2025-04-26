use config::{Config, File, FileFormat};

pub trait Sink {
    fn get_sub_list(&self) -> Vec<String>;
}

pub struct YamlSink {
    sub_list: Vec<String>,
}

impl YamlSink {
    pub fn new(path: &str) -> Self {
        let config = Config::builder()
            .add_source(File::new(path, FileFormat::Yaml))
            .build()
            .unwrap();
        let sub_list = config.get::<Vec<String>>("subscriptions").unwrap();
        Self { sub_list }
    }
}

impl Sink for YamlSink {
    fn get_sub_list(&self) -> Vec<String> {
        self.sub_list.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_yaml_sink() {
        let sink = YamlSink::new("config.yml");
        assert_eq!(
            sink.get_sub_list(),
            vec!["AAPL.US", "TSLA.US", "MSFT.US", "700.HK"]
        );
    }
}
