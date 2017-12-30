#![feature(slice_patterns)]


#[macro_use]
extern crate scad_dots;
#[macro_use]
extern crate scad_dots_derive;
extern crate svg;
extern crate csv;
#[macro_use]
extern crate failure;

pub mod unit;
pub mod spec;
pub mod load;
pub mod catmullrom;
//mod spline;
mod render_3d;
mod render_2d;

use std::path::Path;
use load::read_data;
use failure::Error;


fn main() {
    match read_data(Path::new("data.csv")) {
        Ok(data) => {
            println!("{:?}", data);
            println!("ok");
        }
        Err(err) => print_error(err)
    }
}

fn print_error(error: Error) {
    let mut causes = error.causes();
    if let Some(first) = causes.next() {
        println!("\nError: {}", first);
    }
    for cause in causes {
        println!("Cause: {}", cause);
    }
}
