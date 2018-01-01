#![feature(slice_patterns)]


extern crate csv;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate scad_dots;
#[macro_use]
extern crate scad_dots_derive;
extern crate svg;

pub mod unit;
pub mod spec;
pub mod load;
pub mod catmullrom;
mod spline;
mod station;
mod render_3d;
mod render_2d;

use std::path::Path;
use load::read_data;
use failure::Error;

// use scad_dots::harness::{preview_model};

// TODO: temp
use spec::Spec;
use spec::Config;
use render_2d::SvgDoc;


fn main() {
    match read_data(Path::new("data.csv")) {
        Ok(data) => {
            let resolution = 10;
            let spec = Spec {
                config: Config { stuff: 0 },
                data: data,
            };
            let hull = spec.get_hull(resolution).unwrap();
            let mut doc = SvgDoc::new();

            doc.append_paths(hull.draw_half_breadths());
            doc.save("out.svg").unwrap();

            println!("ok");
        }
        Err(err) => print_error(err),
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
