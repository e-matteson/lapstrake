extern crate lapstrake;

use lapstrake::render_2d::SvgDoc;
use lapstrake::{load_spec, try};
use std::path::Path;

fn main() {
    try(|| {
        // Load the hull.
        let spec = load_spec(Path::new("."))?;
        let hull = spec.get_hull()?;
        let scale_from_feet = 1. / 12.;

        // Show the half-breadth curves for each station/cross-section.
        let mut doc = SvgDoc::new();
        doc.append_vec(hull.draw_half_breadths()?);
        doc.save("images/half-breadth.svg", scale_from_feet)?;

        // Show cross-section templates
        hull.draw_cross_sections(&["Stem".into(), "Post".into()])?
            .save("images/cross-sections.svg", scale_from_feet)?;

        Ok(())
    })
}
