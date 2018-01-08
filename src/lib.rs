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
mod plank;
mod hull;
mod render_3d;

pub use load::load_spec;
pub use render_3d::view_3d;
pub mod render_2d;
pub use spec::*;
pub use hull::*;

use std::process;
use failure::Error;
use util::print_error;

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
