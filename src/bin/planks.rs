extern crate lapstrake;

use std::path::Path;
use lapstrake::{load_spec, try};
use lapstrake::render_2d::SvgDoc;

fn main() {
    try(|| {
        // Load the hull & get its planks.
        let spec = load_spec(Path::new(""))?;
        let hull = spec.get_hull()?;

        // Render the flattened planks to an svg.
        let mut doc = SvgDoc::new();
        for plank in &hull.get_flattened_planks()? {
            doc.append_path(plank.render_2d());
        }
        doc.save("images/plank.svg")?;
        Ok(())
    })
}
