//! Compute [cubic centripetal Catmull-Rom splines](https://en.wikipedia.org/wiki/Centripetal_Catmull%E2%80%93Rom_spline).

use scad_dots::utils::P3;
use scad_dots::utils::distance;

/// A nice cubic interpolation between four points.
pub struct CentripetalCatmullRom {
    // The four points to interpolate between.
    points: [P3; 4],
    // The time parameters at which each of the four points will be hit.
    knots: [f32; 4],
}

/// Which segment of the spline to look at.
/// Whenever possible, use the middle segment.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Segment {
    First,
    Middle,
    Last,
}

impl Segment {
    fn index(&self) -> usize {
        match *self {
            Segment::First => 0,
            Segment::Middle => 1,
            Segment::Last => 2,
        }
    }
}

impl CentripetalCatmullRom {
    /// Construct a Centripetal Catmull-Rom Spline along the four
    /// given points. It is best to sample from the inner segment. The
    /// outer two points are meant to be control points.  (However,
    /// for our rendering we sometimes do need to sample from the
    /// outer segments, so we give that as an option, and hack up an
    /// answer in that case.)
    pub fn new(points: [P3; 4]) -> CentripetalCatmullRom {
        fn knot(points: &[P3; 4], i: usize, prev_knot: f32) -> f32 {
            // 'centripetal' means alpha = 1/2, so take sqrt.
            f32::sqrt(distance(&points[i], &points[i - 1])) + prev_knot
        }

        // Compute knots
        let t_0 = 0.0;
        let t_1 = knot(&points, 1, t_0);
        let t_2 = knot(&points, 2, t_1);
        let t_3 = knot(&points, 3, t_2);
        let knots = [t_0, t_1, t_2, t_3];
        CentripetalCatmullRom {
            points: points,
            knots: knots,
        }
    }

    /// Sample `resolution` points along the chosen segment of the spline.
    /// (Or `resolution + 1` points if `at_end` is true.)
    pub fn sample(
        &self,
        segment: Segment,
        resolution: usize,
        at_end: bool,
    ) -> Vec<P3> {
        let mut samples = vec![];
        for k in 0..resolution {
            let t = k as f32 / resolution as f32;
            samples.push(self.at(t, segment))
        }
        if at_end {
            // If at end of spline, push one extra point.
            // E.g. if sampling from 3 segments of a spline with
            // resolution 2, you want 2 + 2 + 3 = 7 points.
            samples.push(self.at(1.0, segment));
        }
        samples
    }

    // Get the point on the spline a fraction `f` along the given segment.
    fn at(&self, f: f32, segment: Segment) -> P3 {
        let i = segment.index();
        let t = self.knots[i] + f * (self.knots[i + 1] - self.knots[i]);
        self.compute(t, segment != Segment::Middle)
    }

    // Get the point on the spline a fraction `t` along the full curve.
    fn compute(&self, t: f32, use_lagrangian: bool) -> P3 {
        let a_1 = self.intermediate(0, 1, self.points[0], self.points[1], t);
        let a_2 = self.intermediate(1, 2, self.points[1], self.points[2], t);
        let a_3 = self.intermediate(2, 3, self.points[2], self.points[3], t);
        let b_1 = self.intermediate(0, 2, a_1, a_2, t);
        let b_2 = self.intermediate(1, 3, a_2, a_3, t);

        if use_lagrangian {
            // We're not at the middle segment.
            // Catmull-rom splines do not handle this case.
            // We're not really sure how to handle this case well.
            // Let's just fall back to the Lagrange curve.
            self.intermediate(0, 3, b_1, b_2, t)
        } else {
            self.intermediate(1, 2, b_1, b_2, t)
        }
    }

    // The secret sauce.
    fn intermediate(&self, i: usize, j: usize, p: P3, q: P3, t: f32) -> P3 {
        let t_i = self.knots[i];
        let t_j = self.knots[j];
        let left = (t_j - t) / (t_j - t_i) * p;
        let right = (t - t_i) / (t_j - t_i) * q;
        P3::from_coordinates(left.coords + right.coords)
    }
}
