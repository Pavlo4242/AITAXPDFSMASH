#[macro_use]
extern crate log;

pub trait Producer {
    fn next(&mut self) -> Result<Option<Vec<u8>>, String>;
    fn size(&self) -> usize;
}

pub mod dictionary;
pub mod number_ranges;
pub mod custom_query;
pub mod dates;
pub mod default_query;
