extern crate lapstrake;
#[macro_use]
extern crate scad_dots;

use std::path::Path;
use lapstrake::{load_spec, preview_model, try, Tree};

fn main() {
    try(|| {
        // Load the hull.
        let spec = load_spec(Path::new(""))?;
        let hull = spec.get_hull()?;

        // Get renderings for the planks.
        let mut plank_renderings = vec![];
        for plank in &hull.get_planks()? {
            plank_renderings.push(plank.render_3d()?);
        }

        // Render the planks & hull stations
        preview_model(&union![
            Tree::Union(plank_renderings),
            hull.render_stations()?,
        ])?;

        Ok(())
    })
}
