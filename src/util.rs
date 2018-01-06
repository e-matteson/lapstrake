use failure::Error;
use scad_dots::utils::{distance, Axis, P2, P3};

// How near points must be to be considered equal, in feet.
pub const EQUALITY_THRESHOLD: f32 = 0.05;

pub fn practically_zero(x: f32) -> bool {
    f32::abs(x) < 0.00000001
}

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

pub fn remove_duplicates(points: Vec<P3>) -> Vec<P3> {
    let mut good_points = vec![];
    good_points.push(points[0]);
    let mut prev_point = points[0];
    for &point in &points[1..] {
        if distance(&point, &prev_point) >= EQUALITY_THRESHOLD {
            good_points.push(point);
        }
        prev_point = point;
    }
    good_points
}

pub fn reflect2(axis: Axis, points: &[P2]) -> Vec<P2> {
    points
        .iter()
        .map(|p| {
            let mut new = *p;
            new[axis.index()] *= -1.;
            new
        })
        .collect()
}

pub fn reflect3(axis: Axis, points: &[P3]) -> Vec<P3> {
    points
        .iter()
        .map(|p| {
            let mut new = *p;
            new[axis.index()] *= -1.;
            new
        })
        .collect()
}

pub fn print_error(error: Error) {
    let mut causes = error.causes();
    if let Some(first) = causes.next() {
        println!("\nError: {}", first);
    }
    for cause in causes {
        println!("Cause: {}", cause);
    }
}
