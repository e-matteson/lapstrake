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

mod unit;
mod spec;
mod load;
mod catmullrom;
mod util;
mod spline;
mod hull;
mod render_3d;
mod render_2d;

pub use util::print_error;
pub use load::load_spec;
pub use render_2d::SvgDoc;
pub use scad_dots::core::Tree;
pub use scad_dots::harness::preview_model;
pub use hull::FlattenedPlank;

use std::process;
use failure::Error;

pub fn try<F>(run: F)
where
    F: Fn() -> Result<(), Error>,
{
    if let Err(e) = run() {
        print_error(e);
        process::exit(1);
    }
    println!("Done.");
}
