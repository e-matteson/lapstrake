extern crate lapstrake;

use lapstrake::render_2d::SvgDoc;
use lapstrake::{load_spec, try};
use std::path::Path;

fn main() {
    try(|| {
        // Load the hull & get its planks.
        let spec = load_spec(Path::new("."))?;
        let hull = spec.get_hull()?;
        let scale_from_feet = 1. / 12.;

        // Render the flattened planks to an svg.
        let mut doc = SvgDoc::new();
        for plank in &hull.get_flattened_planks()? {
            doc.append(plank.render_2d());
        }
        doc.save("images/plank.svg", scale_from_feet)?;
        Ok(())
    })
}
