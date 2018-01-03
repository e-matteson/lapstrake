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
use unit::Feet;
use spec::Spec;
use hull::FlattenedPlank;
use render_2d::SvgDoc;
use render_3d::{PathStyle3, ScadPath, SCAD_STROKE};
use scad_dots::core::Tree;
use scad_dots::harness::preview_model;

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

    // Show all planks overlayed on each other
    let planks = hull.get_planks()?;

    let mut plank_renderings = vec![];
    for plank in &planks {
        plank_renderings.push(plank.render_3d()?);
    }

    let flattened_planks: Result<Vec<FlattenedPlank>, Error> =
        planks.iter().map(|plank| plank.flatten()).collect();

    let mut doc = SvgDoc::new();
    for plank in &flattened_planks? {
        doc.append_path(plank.render_2d());
    }
    doc.save("images/plank.svg")?;

    /*
    preview_model(&union![
        Tree::Union(plank_renderings),
        hull.render_stations()?,
        hull.render_station_at(Feet::parse("27-4-4")?)?
    ])?;
*/

    Ok(())
}
