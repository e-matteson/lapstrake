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
    pub fn new(points: [P3; 4]) -> CentripetalCatmullRom {
        // Compute knots
        let t_0 = 0.0;
        let t_1 = knot(&points, 1, t_0);
        let t_2 = knot(&points, 2, t_1);
        let t_3 = knot(&points, 3, t_2);
        let knots = [t_0, t_1, t_2, t_3];
        let c = CentripetalCatmullRom {
            points: points,
            knots: knots,
        };
        /*
        println!("");
        for i in 0..4 {
            println!("{} {}", points[i], c.compute(knots[i]));
        }
        */
        c
    }

    /// Sample `resolution` points along the chosen segment of the spline.
    pub fn sample(
        &self,
        segment: Segment,
        resolution: usize,
        at_end: bool,
    ) -> Vec<P3> {
        let mut samples = vec![];
        for k in 0..resolution {
            let t = k as f32 / resolution as f32;
            samples.push(self.at_segment(t, segment))
        }
        if at_end {
            samples.push(self.at_segment(1.0, segment));
        }
        samples
    }

    fn at_segment(&self, f: f32, segment: Segment) -> P3 {
        let i = segment.index();
        let t = self.knots[i] + f * (self.knots[i + 1] - self.knots[i]);
        self.compute(t, segment != Segment::Middle)
    }

    fn compute(&self, t: f32, at_end: bool) -> P3 {
        // Compute intermediates
        let a_1 = self.intermediate(0, 1, self.points[0], self.points[1], t);
        let a_2 = self.intermediate(1, 2, self.points[1], self.points[2], t);
        let a_3 = self.intermediate(2, 3, self.points[2], self.points[3], t);
        let b_1 = self.intermediate(0, 2, a_1, a_2, t);
        let b_2 = self.intermediate(1, 3, a_2, a_3, t);
        // Compute answer

        if at_end {
            // We're not at the middle segment.
            // Catmull-rom splines do not handle this case.
            // We're not really sure how to handle this case well.
            // Let's just fall back to the Lagrange curve.
            self.intermediate(0, 3, b_1, b_2, t)
        } else {
            self.intermediate(1, 2, b_1, b_2, t)
        }
    }

    fn intermediate(&self, i: usize, j: usize, p: P3, q: P3, t: f32) -> P3 {
        let t_i = self.knots[i];
        let t_j = self.knots[j];
        let left = (t_j - t) / (t_j - t_i) * p;
        let right = (t - t_i) / (t_j - t_i) * q;
        P3::from_coordinates(left.coords + right.coords)
    }
}


fn knot(points: &[P3; 4], i: usize, prev_knot: f32) -> f32 {
    // 'centripetal' means alpha = 1/2, so take sqrt.
    f32::sqrt(distance(&points[i], &points[i - 1])) + prev_knot
}
