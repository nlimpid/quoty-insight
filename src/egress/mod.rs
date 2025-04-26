pub mod influxdb;

pub trait Egress {
    fn send(&self, data: &str);
}

/// An Egress implementation that prints data to standard output.
pub struct PrintEgress;

impl Egress for PrintEgress {
    fn send(&self, data: &str) {
        println!("Egress data: {:?}", data);
    }
}
