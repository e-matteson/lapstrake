use scad_dots::utils::{Axis, P2, P3};


pub fn project_points(axis: Axis, points: &[P3]) -> Vec<P2> {
    points.iter().map(|&p| project(axis, p)).collect()
}

pub fn project(axis: Axis, point: P3) -> P2 {
    match axis {
        Axis::X => P2::new(point.y, point.z),
        Axis::Y => P2::new(point.x, point.z),
        Axis::Z => P2::new(point.x, point.y),
    }
}
