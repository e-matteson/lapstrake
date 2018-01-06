extern crate lapstrake;

use std::iter;
use std::path::Path;
use lapstrake::{load_spec, try, view_3d};

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
        let renderings = plank_renderings
            .into_iter()
            .chain(iter::once(hull.render_stations()?))
            .collect();

        view_3d(renderings)?;

        Ok(())
    })
}
