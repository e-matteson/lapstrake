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
mod station;
mod render_3d;
mod render_2d;

use std::process;
use std::path::Path;
use load::read_data;
use failure::Error;

use unit::Feet;
use spec::Spec;
use spec::Config;
use render_2d::SvgDoc;


const STATION_RESOLUTION: usize = 20;
const PLANK_RESOLUTION: usize = 1;
const NUM_PLANKS: usize = 10;
const OVERLAP: Feet = Feet {
    feet: 0,
    inches: 3,
    eighths: 0,
};

fn main() {
    let data = check_error(read_data(Path::new("data.csv")));
    let spec = Spec {
        config: Config { stuff: 0 },
        data: data,
    };
    let hull = check_error(spec.get_hull(STATION_RESOLUTION));

    // Show the half-breadth curves for each station/cross-section
    let mut doc = SvgDoc::new();
    doc.append_paths(hull.draw_half_breadths());
    check_error(doc.save("half-breadth.svg"));

    // Show all planks overlayed on each other
    let planks = check_error(hull.get_planks(
        NUM_PLANKS,
        OVERLAP.into(),
        PLANK_RESOLUTION,
    ));
    let flattened_planks: Vec<_> = planks
        .iter()
        .map(|plank| check_error(plank.flatten()))
        .collect();
    let mut doc = SvgDoc::new();
    for plank in &flattened_planks {
        doc.append_path(plank.render_2d());
    }
    check_error(doc.save("plank.svg"));

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
