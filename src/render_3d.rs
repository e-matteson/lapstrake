use std::f32::consts::PI;

use failure::Error;

use scad_dots::core::{chain, mark, Dot, DotAlign, DotSpec, Shape, Tree};
use scad_dots::utils::{axis_radians, Corner3 as C3, P3, R3, V3};
use scad_dots::harness::{check_model, Action};
// use scad_dots::parse::scad_relative_eq;


/// Example:
///
/// ```
/// let path = ScadPath::new(vec![
///     P3::new(0., 0., 0.),
///     P3::new(50., 50., 0.),
///     P3::new(100., 20., 75.),
/// ]).show_points();
///
/// preview_model(&path.link(PathStyle3::Line)?)?;
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

impl ScadPath {
    pub fn new(points: Vec<P3>) -> ScadPath {
        ScadPath {
            points: points,
            show_points: false,
            stroke: 1.,
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

    pub fn link(self, style: PathStyle3) -> Result<Tree, Error> {
        let dots = self.make_dots(self.stroke);
        let mut tree = match style {
            PathStyle3::Dots => Tree::Union(dots),
            PathStyle3::Solid => Tree::Hull(dots),
            PathStyle3::Line => chain(&dots)?,
        };
        if self.show_points {
            let markers = Tree::Union(self.make_dots(self.stroke * 2.));
            tree = union![tree, markers];
        }
        Ok(tree)
    }


    fn make_dots(&self, diameter: f32) -> Vec<Tree> {
        let mut dots = Vec::new();
        for p in &self.points {
            let spec = DotSpec {
                pos: p.to_owned(),
                align: DotAlign::center_solid(),
                size: diameter,
                rot: R3::identity(),
            };
            dots.push(dot![Dot::new(Shape::Sphere, spec)]);
        }
        dots
    }
}



#[test]
fn test_path_surface() {
    check_model("test_path_surface", Action::Preview, || {
        let path = ScadPath::new(vec![
            P3::new(0., 0., 20.),
            P3::new(0., 10., 0.),
            P3::new(0., 5., -10.),
        ]).show_points();
        path.link(PathStyle3::Solid)
    })
}


#[test]
fn test_path_line_dots() {
    check_model("test_path_line_dots", Action::Preview, || {
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
        let n = Dot::new(
            Shape::Sphere,
            DotSpec {
                pos: P3::origin(),
                align: C3::P000.into(),
                size: 2.0,
                rot: axis_radians(V3::x_axis().unwrap(), PI / 4.),
            },
        );
        Ok(dot![n])
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
