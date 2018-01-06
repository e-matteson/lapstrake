//! A spline with any number of points.
//!
//! Implemented with the centripetal Catmull-Rom algorithm.

use std::iter;

use util::practically_zero;
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
        let ref_points: Vec<(P3, usize)> = count_multiplicity(ref_points);
        let n = ref_points.len();
        if n < 4 {
            bail!("Splines must have at least 4 points.")
        }
        let mut points = vec![];
        for i in 0..n - 3 {
            let array = [
                ref_points[i].0,
                ref_points[i + 1].0,
                ref_points[i + 2].0,
                ref_points[i + 3].0,
            ];
            let catmull = CentripetalCatmullRom::new(array);
            if i == 0 {
                points.extend(repeat(ref_points[i], resolution));
                points.extend(catmull.sample(First, resolution, false));
            }
            points.extend(repeat(ref_points[i + 1], resolution));
            points.extend(catmull.sample(Middle, resolution, false));
            if i == n - 4 {
                points.extend(repeat(ref_points[i + 2], resolution));
                points.extend(catmull.sample(Last, resolution, true));
                points.extend(repeat(ref_points[i + 3], resolution));
            }
        }
        Ok(Spline { points: points })
    }

    /// A sample of points along the spline, at the resolution given
    /// at construction.
    pub fn sample(&self, resolution: Option<usize>) -> Vec<P3> {
        match resolution {
            None => self.points.clone(),
            Some(resolution) => {
                let mut points = vec![];
                for i in 0..resolution + 1 {
                    let t = i as f32 / resolution as f32;
                    points.push(self.at_t(t));
                }
                points
            }
        }
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
    pub fn at_len(&self, desired_length: f32) -> P3 {
        let mut length = 0.0;
        let mut prev_point = self.points[0];
        for &point in &self.points[1..] {
            let delta = projected_distance(Axis::X, point, prev_point);
            if length + delta >= desired_length {
                // We are between prev_point and point. Linearly interpolate.
                // The projection throws this off a bit, but it shouldn't matter.
                if practically_zero(delta) {
                    return prev_point;
                } else {
                    let t = (desired_length - length) / delta;
                    return linear_interpolate(t, prev_point, point);
                }
            } else {
                length += delta;
                prev_point = point;
            }
        }
        panic!("Fell off the end of a spline.");
    }

    // Get the point at a given fraction along the curve.
    pub fn at_t(&self, t: f32) -> P3 {
        let len = self.length();
        self.at_len(t * len)
    }

    // Get the point at a given x coordinate (a.k.a. position).
    pub fn at_x(&self, desired_x: f32) -> Result<P3, Error> {
        let i = match self.points
            .binary_search_by(|pt| pt.x.partial_cmp(&desired_x).unwrap())
        {
            Ok(i) => i,
            Err(i) => i,
        };
        Ok(*self.points
            .get(i)
            .expect(&format!("Could not get point at position {}", desired_x)))
    }
}

fn linear_interpolate(t: f32, pt1: P3, pt2: P3) -> P3 {
    P3::from_coordinates((1.0 - t) * pt1.coords + t * pt2.coords)
}

fn projected_distance(axis: Axis, point_a: P3, point_b: P3) -> f32 {
    let v = project(axis, point_b) - project(axis, point_a);
    (v.x.powf(2.) + v.y.powf(2.)).sqrt()
}

// Catmull-Rom doesn't play well with equal points.
// Compute the multiplicity of each point in the path so we can work
// around this.
fn count_multiplicity(points: Vec<P3>) -> Vec<(P3, usize)> {
    let mut answer = vec![];
    let mut prev_point = *points.get(0).expect("Spline given empty vector.");
    let mut multiplicity = 1;
    for &point in &points[1..] {
        if point == prev_point {
            multiplicity += 1;
        } else {
            answer.push((prev_point, multiplicity));
            multiplicity = 1;
        }
        prev_point = point;
    }
    answer.push((prev_point, multiplicity));
    answer
}

// Catmull-Rom doesn't play well with equal points.
// If we find equal points, just "interpolate" the same point over and over.
fn repeat(point_with_multiplicity: (P3, usize), resolution: usize) -> Vec<P3> {
    let (point, multiplicity) = point_with_multiplicity;
    let len = (multiplicity - 1) * resolution;
    iter::repeat(point).take(len).collect()
}
