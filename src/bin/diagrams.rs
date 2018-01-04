extern crate lapstrake;

use std::path::Path;
use lapstrake::{load_spec, try, SvgDoc};

fn main() {
    try(|| {
        // Load the hull.
        let spec = load_spec(Path::new(""))?;
        let hull = spec.get_hull()?;

        // Show the half-breadth curves for each station/cross-section.
        let mut doc = SvgDoc::new();
        doc.append_paths(hull.draw_half_breadths());
        doc.save("images/half-breadth.svg")?;
        Ok(())
    })
}
