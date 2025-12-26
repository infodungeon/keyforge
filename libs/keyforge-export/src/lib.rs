pub mod qmk;
pub mod via;
pub mod zmk;

use anyhow::Result;

pub trait Exporter {
    fn generate(&self, layout_name: &str, keys: &[String]) -> Result<String>;
}
