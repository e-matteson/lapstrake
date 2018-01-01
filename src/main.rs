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
use load::{read_config, read_data};
use failure::Error;

use spec::Spec;
use hull::FlattenedPlank;
use render_2d::SvgDoc;
use render_3d::{PathStyle3, ScadPath};
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
    let config = read_config(Path::new("config.csv"))?;
    let data = read_data(Path::new("data.csv"))?;
    let spec = Spec {
        config: config,
        data: data,
    };
    let hull = spec.get_hull()?;

    // Show the half-breadth curves for each station/cross-section
    let mut doc = SvgDoc::new();
    doc.append_paths(hull.draw_half_breadths());
    doc.save("images/half-breadth.svg")?;

    // Show all planks overlayed on each other
    let planks = hull.get_planks()?;

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
    doc.save("images/plank.svg")?;

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
