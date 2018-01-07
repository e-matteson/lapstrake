extern crate lapstrake;

use std::path::Path;
use lapstrake::{load_spec, try};
use lapstrake::render_2d::SvgDoc;

fn main() {
    try(|| {
        // Load the hull.
        let spec = load_spec(Path::new(""))?;
        let hull = spec.get_hull()?;

        // Show the half-breadth curves for each station/cross-section.
        let mut doc = SvgDoc::new();
        doc.append_vec(hull.draw_half_breadths());
        doc.save("images/half-breadth.svg")?;

        // Show cross-section templates
        hull.draw_cross_sections(&["Stem".into(), "Post".into()])?.save("images/cross-sections.svg")?;
        Ok(())
    })
}
