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

use std::path::Path;
use load::read_data;
use failure::Error;

use unit::Feet;
use spec::Spec;
use spec::Config;
use station::FlattenedPlank;
use render_2d::SvgDoc;
use render_3d::{PathStyle3, ScadPath};
use scad_dots::core::Tree;
use scad_dots::harness::preview_model;


const STATION_RESOLUTION: usize = 20;
const PLANK_RESOLUTION: usize = 1;
const NUM_PLANKS: usize = 10;
const OVERLAP: Feet = Feet {
    feet: 0,
    inches: 3,
    eighths: 0,
};

fn main() {
    if let Err(e) = run() {
        print_error(e);
        ::std::process::exit(1);
    }
    println!("ok");
}

fn run() -> Result<(), Error> {
    let data = read_data(Path::new("data.csv"))?;
    let spec = Spec {
        config: Config { stuff: 0 },
        data: data,
    };
    let hull = spec.get_hull(STATION_RESOLUTION)?;

    // Show the half-breadth curves for each station/cross-section
    let mut doc = SvgDoc::new();
    doc.append_paths(hull.draw_half_breadths());
    doc.save("half-breadth.svg")?;

    // Show all planks overlayed on each other
    let planks = hull.get_planks(NUM_PLANKS, OVERLAP.into(), PLANK_RESOLUTION)?;

    let mut tops = Vec::new();
    let mut bottoms = Vec::new();
    let mut dots = Vec::new();

    for plank in &planks[0..1] {
        tops.push(ScadPath::new(plank.top_line.sample())
            .stroke(15.)
            .link(PathStyle3::Line)?);
        bottoms.push(ScadPath::new(plank.bottom_line.sample())
            .stroke(5.)
            .link(PathStyle3::Line)?);
    }

    for plank in &planks {
        dots.push(ScadPath::new(plank.outline())
            .stroke(15.)
            .link(PathStyle3::Dots)?);
    }

    preview_model(&union![
        Tree::Union(dots),
        Tree::Union(tops),
        Tree::Union(bottoms),
        hull.render_stations()?,
    ])?;

    let flattened_planks: Result<Vec<FlattenedPlank>, Error> =
        planks.iter().map(|plank| plank.flatten()).collect();

    let mut doc = SvgDoc::new();
    for plank in &flattened_planks? {
        doc.append_path(plank.render_2d());
    }
    doc.save("plank.svg")?;

    Ok(())
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
