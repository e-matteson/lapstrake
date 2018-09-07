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
#[macro_use]
extern crate structopt;

mod catmullrom;
mod draw;
mod error;
mod hull;
mod load;
mod plank;
pub mod render_2d; // only public because of doctests - can we avoid that?
pub mod render_3d;
mod spec;
mod spline;
mod unit;
mod util;

// pub use draw::*;
// pub use hull::*;
// pub use load::load_spec;
pub use render_3d::preview_model;

use std::path::Path;

use structopt::StructOpt;

use error::LapstrakeError;
// use load::load_spec;
// use render_2d::SvgDoc;
pub use spec::Spec;

/// Tool for model-ship building
#[derive(StructOpt, Debug)]
#[structopt(name = "lapstrake")]
struct Options {
    /// The scale factor from the input dimensions to the output dimensions.
    #[structopt(short = "s", long = "scale")]
    scale: Option<f32>,

    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    /// Display a 3d model of the hull's stations and plank edges
    #[structopt(name = "wireframe")]
    Wireframe,

    /// Output various 2d diagrams of the hull shape as .svg files.
    #[structopt(name = "diagrams")]
    Diagrams,

    /// Output 2d cross-sections of the hull (stations) to a .svg, suitable for assembling into a frame.
    #[structopt(name = "stations")]
    Stations,

    /// Output 2d shapes of planks to a .svg, according to the specification in the planks spreadsheet.
    #[structopt(name = "planks")]
    Planks,
}

fn run() -> Result<(), LapstrakeError> {
    let options = Options::from_args();

    let output_folder = Path::new("./output");
    let input_folder = Path::new("./input");

    let spec = Spec::load_from(input_folder)?;
    let hull = spec.get_hull()?;
    let scale = options.scale.unwrap_or(1.);

    let output_to = |filename: &str| {
        let mut path = output_folder.to_owned();
        path.push(filename);
        path
    };

    match options.command {
        Command::Wireframe => preview_model(&hull.render_half_wireframe()?)?,
        Command::Diagrams => hull
            .draw_half_breadths()?
            .save(&output_to("half-breadths.svg"), scale)?,
        Command::Stations => hull
            .draw_cross_sections(&["Stem".into(), "Post".into()])?
            .save(&output_to("stations.svg"), scale)?,
        Command::Planks => {
            hull.draw_planks()?.save(&output_to("planks.svg"), scale)?
        }
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        println!("{}", error);
        ::std::process::exit(1);
    } else {
        println!("Done.");
    }
}
