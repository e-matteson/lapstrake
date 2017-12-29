#![feature(slice_patterns)]

extern crate svg;
extern crate csv;
#[macro_use]
extern crate failure;

mod unit;
mod spec;
mod load;

use std::path::Path;
use load::read_data;
use load::print_error;

fn main() {
    match read_data(Path::new("data.csv")) {
        Ok(data) => {
            println!("{:?}", data);
            println!("ok");
        }
        Err(err) => print_error(err)
    }
}
