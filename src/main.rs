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

use scad_dots::core::{chain, mark, Dot, DotAlign, DotSpec, Shape, Tree};
use scad_dots::utils::{axis_radians, Corner3 as C3, P3, R3, V3};
use scad_dots::harness::{check_model, preview_model, Action};

// TODO: temp
use spec::Spec;
use spec::Config;
use render_2d::SvgDoc;


fn main() {
    match read_data(Path::new("data.csv")) {
        Ok(data) => {
            let resolution = 2;
            let spec = Spec {
                config: Config { stuff: 0 },
                data: data,
            };
            let hull = spec.get_hull(resolution).unwrap();
            //            hull.stations[3].render_3d();
            let mut doc = SvgDoc::new();

            for station in &hull.stations {
                doc.append_path(station.render_spline_2d());
                doc.append_path(station.render_points_2d());
            }
            doc.save("out.svg");
            // let mut trees = Vec::new();
            //     trees.push(station.render_3d().unwrap());
            // preview_model(&Tree::Union(trees)).unwrap();

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
