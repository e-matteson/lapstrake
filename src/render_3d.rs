use std::f32::consts::PI;

use scad_dots::core::{mark, Dot, DotSpec, Shape, Tree};
use scad_dots::utils::{axis_radians, Corner3 as C3, P3, V3};
use scad_dots::harness::{check_model, Action};
// use scad_dots::parse::scad_relative_eq;


#[test]
fn test_dot_sphere() {
    check_model("test_dot_sphere", Action::Run, || {
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
    check_model("test_surface", Action::Run, || {
        Ok(hull![
            mark(P3::origin(), 1.),
            mark(P3::new(10., 0., 0.,), 1.),
            mark(P3::new(10., 10., 0.), 1.)
        ])
    })
}
