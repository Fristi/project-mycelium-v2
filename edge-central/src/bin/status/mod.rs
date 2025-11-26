pub mod i2c;

use anyhow::Result;

pub trait Status {
    fn show(&mut self, small_text: &str, big_text: &str) -> Result<()>;
}