#![feature(slice_patterns)]


#[macro_use]
extern crate scad_dots;
#[macro_use]
extern crate scad_dots_derive;
extern crate svg;
extern crate csv;
#[macro_use]
extern crate failure;

mod unit;
mod spec;
mod load;
mod render_3d;
mod render_2d;

use render_2d::{SvgColor, SvgPath};
use scad_dots::utils::P2;

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
