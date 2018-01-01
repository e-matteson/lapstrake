#![feature(slice_patterns)]

extern crate csv;
#[macro_use]
extern crate failure;
extern crate nalgebra;
#[macro_use]
extern crate scad_dots;
#[macro_use]
extern crate scad_dots_derive;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate svg;

pub mod unit;
pub mod spec;
pub mod load;
pub mod catmullrom;
mod spline;
mod hull;
mod render_3d;
mod render_2d;

use std::process;
use std::path::Path;
use load::{read_config, read_data};
use failure::Error;

use spec::Spec;
use render_2d::SvgDoc;

fn main() {
    let config = check_error(read_config(Path::new("config.csv")));
    let data = check_error(read_data(Path::new("data.csv")));
    let spec = Spec {
        config: config,
        data: data,
    };
    let hull = check_error(spec.get_hull());

    // Show the half-breadth curves for each station/cross-section
    let mut doc = SvgDoc::new();
    doc.append_paths(hull.draw_half_breadths());
    check_error(doc.save("images/half-breadth.svg"));

    // Show all planks overlayed on each other
    let planks = check_error(hull.get_planks());
    let flattened_planks: Vec<_> = planks
        .iter()
        .map(|plank| check_error(plank.flatten()))
        .collect();
    let mut doc = SvgDoc::new();
    for plank in &flattened_planks {
        doc.append_path(plank.render_2d());
    }
    check_error(doc.save("images/plank.svg"));

    println!("ok");
}

fn check_error<T>(result: Result<T, Error>) -> T {
    match result {
        Ok(ans) => ans,
        Err(err) => {
            print_error(err);
            process::exit(1);
        }
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
