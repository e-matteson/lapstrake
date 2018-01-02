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
mod util;
mod spline;
mod hull;
mod render_3d;
mod render_2d;

use std::path::Path;
use load::{read_config, read_data, read_planks};
use failure::Error;

use util::print_error;
use spec::Spec;
// use hull::FlattenedPlank;
use render_2d::SvgDoc;
// use render_3d::{PathStyle3, ScadPath, SCAD_STROKE};
// use scad_dots::core::Tree;
// use scad_dots::harness::preview_model;

fn main() {
    if let Err(e) = run() {
        print_error(e);
        ::std::process::exit(1);
    }
    println!("done");
}

fn run() -> Result<(), Error> {
    let data = read_data(Path::new("data.csv"))?;
    let planks = read_planks(Path::new("planks.csv"))?;
    let config = read_config(Path::new("config.csv"))?;
    let spec = Spec {
        data: data,
        planks: planks,
        config: config,
    };
    let hull = spec.get_hull()?;

    // Show the half-breadth curves for each station/cross-section
    let mut doc = SvgDoc::new();
    doc.append_paths(hull.draw_half_breadths());
    doc.save("images/half-breadth.svg")?;

    // Show the closed cross-sections for each station.
    hull.draw_cross_sections(&["Stem".into(), "Post".into()])?
        .save("images/cross-sections.svg")?;

    Ok(())
}
