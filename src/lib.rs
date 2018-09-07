// #![feature(slice_patterns)]

extern crate csv;
extern crate nalgebra;
#[macro_use]
extern crate scad_dots;
#[macro_use]
extern crate scad_dots_derive;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate svg;

mod catmullrom;
mod draw;
mod error;
mod hull;
mod load;
mod plank;
mod render_3d;
mod spec;
mod spline;
mod unit;
mod util;

pub use load::load_spec;
pub use render_3d::view_3d;
pub mod render_2d;
pub use draw::*;
use error::LapstrakeError;
pub use hull::*;
pub use spec::*;

use std::process;

pub fn try<F>(run: F)
where
    F: Fn() -> Result<(), LapstrakeError>,
{
    if let Err(e) = run() {
        format!("error: {}", e);
        process::exit(1);
    }
    println!("Done.");
}
