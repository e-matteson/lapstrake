//! A spline with any number of points.
//!
//! Implemented with the centripetal Catmull-Rom algorithm.

use scad_dots::utils::P3;
use scad_dots::utils::distance;
use failure::Error;

use catmullrom::CentripetalCatmullRom;
use catmullrom::Segment::{First, Last, Middle};

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
        /*
        println!("!!");
        for pt in &ref_points {
            println!("{} {} {}", pt.x, pt.y, pt.z);
        }
        println!("!!!");
        for pt in &points {
            println!("{} {} {}", pt.x, pt.y, pt.z);
        }
        */
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
        let mut prev_point = self.points[0];
        for &point in &self.points[1..] {
            length += distance(&point, &prev_point);
            prev_point = point;
        }
        length
    }

    /// Get the point at a given distance along the curve from the
    /// start of the spline.
    pub fn at(&self, dist: f32) -> P3 {
        let mut length = 0.0;
        let mut prev_point = self.points[0];
        for &point in &self.points[1..] {
            let delta = distance(&point, &prev_point);
            if length + delta >= dist {
                // We are between prev_point and point. Linearly interpolate.
                let t = (dist - length) / delta;
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
