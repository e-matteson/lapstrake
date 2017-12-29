#![feature(slice_patterns)]


#[macro_use]
extern crate scad_dots;
#[macro_use]
extern crate scad_dots_derive;
extern crate svg;

#[macro_use]
extern crate failure;

mod unit;
mod spec;
mod render_3d;
mod render_2d;

use render_2d::{SvgColor, SvgPath};
use scad_dots::utils::P2;


fn main() {}
