//! A spline with any number of points.
//!
//! Implemented with the centripetal Catmull-Rom algorithm.

use scad_dots::utils::{distance, Axis, P3};
use failure::Error;

use catmullrom::CentripetalCatmullRom;
use catmullrom::Segment::{First, Last, Middle};
use util::{project, project_points};

/// A spline with any number of points.
#[derive(Debug, Clone)]
pub struct Spline {
    points: Vec<P3>,
}

impl Spline {
    pub fn new(
        ref_points: Vec<P3>,
        resolution: usize,
    ) -> Result<Spline, Error> {
        let n = ref_points.len();
        if n < 4 {
            bail!("Splines must have at least 4 points.")
        }
        let mut points = vec![];
        for i in 0..n - 3 {
            let array = [
                ref_points[i],
                ref_points[i + 1],
                ref_points[i + 2],
                ref_points[i + 3],
            ];
            let catmull = CentripetalCatmullRom::new(array);
            if i == 0 {
                points.extend(catmull.sample(First, resolution, false));
            }
            points.extend(catmull.sample(Middle, resolution, false));
            if i == n - 4 {
                points.extend(catmull.sample(Last, resolution, true));
            }
        }
        Ok(Spline { points: points })
    }

    /// A sample of points along the spline, at the resolution given
    /// at construction.
    pub fn sample(&self) -> Vec<P3> {
        self.points.clone()
    }

    /// The total length of the spline.
    pub fn length(&self) -> f32 {
        let mut length = 0.0;

        let flat_points = project_points(Axis::X, &self.points);
        let mut prev_point = flat_points[0];
        for &point in &flat_points[1..] {
            length += distance(&point, &prev_point);
            prev_point = point;
        }
        length
    }

    /// Get the point at a given distance along the curve from the
    /// start of the spline.
    pub fn at(&self, desired_length: f32) -> P3 {
        let mut length = 0.0;
        let mut prev_point = self.points[0];
        for &point in &self.points[1..] {
            let delta = projected_distance(Axis::X, point, prev_point);
            if length + delta >= desired_length {
                // We are between prev_point and point. Linearly interpolate.
                // The projection throws this off a bit, but it shouldn't matter.
                let t = (desired_length - length) / delta;
                return linear_interpolate(t, prev_point, point);
            } else {
                length += delta;
                prev_point = point;
            }
        }
        panic!("Fell off the end of a spline.");
    }
}

fn linear_interpolate(t: f32, pt1: P3, pt2: P3) -> P3 {
    P3::from_coordinates((1.0 - t) * pt1.coords + t * pt2.coords)
}

fn projected_distance(axis: Axis, point_a: P3, point_b: P3) -> f32 {
    let v = project(axis, point_b) - project(axis, point_a);
    (v.x.powf(2.) + v.y.powf(2.)).sqrt()
}
