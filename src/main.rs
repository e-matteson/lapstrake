#![feature(slice_patterns)]


extern crate csv;
#[macro_use]
extern crate failure;
extern crate nalgebra;
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

use unit::Feet;
use spec::Spec;
use spec::Config;
use render_2d::SvgDoc;


const STATION_RESOLUTION: usize = 20;
const PLANK_RESOLUTION: usize = 30;
const NUM_PLANKS: usize = 10;
const OVERLAP: Feet = Feet {
    feet: 0,
    inches: 3,
    eighths: 0,
};

fn main() {
    match read_data(Path::new("data2.csv")) {
        Ok(data) => {
            let spec = Spec {
                config: Config { stuff: 0 },
                data: data,
            };
            let hull = spec.get_hull(STATION_RESOLUTION).unwrap();

            // Show the half-breadth curves for each station/cross-section
            let mut doc = SvgDoc::new();
            doc.append_paths(hull.draw_half_breadths());
            doc.save("half-breadth.svg").unwrap();

            // Show all planks overlayed on each other
            let planks = hull.get_planks(NUM_PLANKS, OVERLAP.into(), PLANK_RESOLUTION)
                .unwrap();
            let flattened_planks: Vec<_> = planks
                .iter()
                .map(|plank| plank.flatten().unwrap())
                .collect();
            let mut doc = SvgDoc::new();
            for plank in &flattened_planks {
                doc.append_path(plank.render_2d());
            }
            doc.save("plank.svg");

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
