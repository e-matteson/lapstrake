use scad_dots::core::{chain, Dot, DotAlign, DotShape, DotSpec, Tree};
use scad_dots::errors::ScadDotsError;
use scad_dots::harness::preview_model;
use scad_dots::utils::{P3, R3};

pub const SCAD_STROKE: f32 = 0.1;

/// Example:
///
/// ```
/// extern crate lapstrake;
/// extern crate scad_dots;
/// use lapstrake::render_3d::{view_3d, PathStyle3, ScadPath};
/// use scad_dots::utils::P3;
///
/// let path = ScadPath::new(vec![
///     P3::new(0., 0., 0.),
///     P3::new(5., 5., 0.),
///     P3::new(10., 2., 7.5),
/// ]).show_points()
/// .link(PathStyle3::Line)
/// .unwrap();
/// ```
pub struct ScadPath {
    points: Vec<P3>,
    show_points: bool,
    stroke: f32,
}

#[allow(dead_code)]
pub enum PathStyle3 {
    Dots,
    Line,
    Solid,
}

pub fn view_3d(renderings: Vec<Tree>) -> Result<(), ScadDotsError> {
    preview_model(&Tree::union(renderings))
}

impl ScadPath {
    pub fn new(points: Vec<P3>) -> ScadPath {
        ScadPath {
            points: points,
            show_points: false,
            stroke: 0.01,
        }
    }

    pub fn show_points(mut self) -> ScadPath {
        self.show_points = true;
        self
    }

    pub fn stroke(mut self, width: f32) -> Self {
        self.stroke = width;
        self
    }

    pub fn link(self, style: PathStyle3) -> Result<Tree, ScadDotsError> {
        let dots = self.make_dots(self.stroke);
        let mut tree = match style {
            PathStyle3::Dots => Tree::union(dots),
            PathStyle3::Solid => Tree::hull(dots),
            PathStyle3::Line => chain(&dots)?,
        };
        if self.show_points {
            let markers = Tree::union(self.make_dots(self.stroke * 2.));
            tree = union![tree, markers];
        }
        Ok(tree)
    }

    fn make_dots(&self, diameter: f32) -> Vec<Tree> {
        let mut dots = Vec::new();
        for p in &self.points {
            let spec = DotSpec {
                pos: p.to_owned(),
                align: DotAlign::centroid(),
                size: diameter,
                rot: R3::identity(),
                shape: DotShape::Sphere,
            };
            dots.push(Dot::new(spec).into());
        }
        dots
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scad_dots::core::mark;
    use scad_dots::harness::{check_model, Action};
    use scad_dots::utils::{axis_radians, Corner3 as C3, V3};
    use std::f32::consts::PI;

    #[test]
    fn test_path_surface() {
        check_model("test_path_surface", Action::Test, || {
            let path = ScadPath::new(vec![
                P3::new(0., 0., 20.),
                P3::new(0., 10., 0.),
                P3::new(0., 5., -10.),
            ]).show_points();
            path.link(PathStyle3::Solid)
            // .map_err(|e| ScadDotsError::External(Box::new(e)))
        })
    }

    #[test]
    fn test_path_line_dots() {
        check_model("test_path_line_dots", Action::Test, || {
            let path = ScadPath::new(vec![
                P3::new(0., 0., 0.),
                P3::new(50., 50., 0.),
                P3::new(100., 20., 75.),
            ]).show_points();
            path.link(PathStyle3::Line)
        })
    }

    #[test]
    fn test_dot_sphere() {
        check_model("test_dot_sphere", Action::Test, || {
            let n = Dot::new(DotSpec {
                pos: P3::origin(),
                align: C3::P000.into(),
                size: 2.0,
                rot: axis_radians(V3::x_axis().unwrap(), PI / 4.),
                shape: DotShape::Sphere,
            });
            Ok(Tree::from(n))
        })
    }

    #[test]
    fn test_surface() {
        check_model("test_surface", Action::Test, || {
            Ok(hull![
                mark(P3::origin(), 1.),
                mark(P3::new(10., 0., 0.,), 1.),
                mark(P3::new(10., 10., 0.), 1.)
            ])
        })
    }
}
