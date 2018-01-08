use std::iter;
use nalgebra::{normalize, Rotation2};
use scad_dots::utils::{Axis, P2, P3, V2};
use scad_dots::core::{MinMaxCoord, Tree};
use failure::Error;

use util::{EQUALITY_THRESHOLD, practically_zero};
use spline::Spline;
use scad_dots::utils::distance;
use render_3d::{PathStyle3, ScadPath, SCAD_STROKE};
use render_2d::{PathStyle2, SvgColor, SvgPath};

/// A plank on the hull.
/// This is a 3d object located at its position on the ship.
#[derive(Debug, Clone)]
pub struct Plank {
    pub top_line: Spline,
    pub bottom_line: Spline,
    pub resolution: usize,
}

/// A flattened plank.  This is a 2d object, taken originally from the
/// hull but not located on a 2d surface.
#[derive(Debug, Clone, MinMaxCoord)]
pub struct FlattenedPlank {
    pub top_line: Vec<P2>,
    pub bottom_line: Vec<P2>,
}

impl FlattenedPlank {
    /// Render as an SVG path.
    pub fn render_2d(&self) -> SvgPath {
        SvgPath::new(self.get_outline())
            .stroke(SvgColor::Black, 0.01)
            .style(PathStyle2::Line)
    }

    fn get_outline(&self) -> Vec<P2> {
        let top_line = self.top_line.clone();
        let mut bottom_line = self.bottom_line.clone();
        bottom_line.reverse();

        let mut points = vec![];
        points.extend(top_line);
        points.extend(bottom_line);
        points.push(self.top_line[0]);
        points
    }

    fn orient_horizontally(&mut self) {
        let left = self.top_line[0];
        let right = self.top_line[self.top_line.len() - 1];
        let angle =
            Rotation2::rotation_between(&(right - left), &V2::new(1.0, 0.0));
        for pt in self.top_line.iter_mut().chain(self.bottom_line.iter_mut()) {
            *pt = left + angle * (*pt - left);
        }
    }

    fn shift_up(&mut self, dist: f32) {
        for pt in self.top_line.iter_mut().chain(self.bottom_line.iter_mut()) {
            pt.y += dist;
        }
    }

    // Flatten planks to 2d. Place them nicely, without overlap.
    pub(crate) fn flatten_planks(planks: Vec<Plank>)
                                 -> Result<Vec<FlattenedPlank>, Error>
    {
        let mut layed_planks = vec![];
        let mut last_y = None;
        for mut plank in planks {
            let mut plank = plank.flatten()?;
            plank.orient_horizontally();
            if let Some(last_y) = last_y {
                let y = plank.min_coord(Axis::Y);
                plank.shift_up(last_y - y + 2.0 * EQUALITY_THRESHOLD);
            }
            last_y = Some(plank.max_coord(Axis::Y));
            layed_planks.push(plank);
        }
        Ok(layed_planks)
    }
}

impl Plank {
    pub(crate) fn new(
        bot_line: Vec<P3>,
        top_line: Vec<P3>,
        resolution: usize,
    ) -> Result<Plank, Error> {
        Ok(Plank {
            resolution: ((bot_line.len() + top_line.len()) / 2) * resolution,
            bottom_line: Spline::new(bot_line, resolution)?,
            top_line: Spline::new(top_line, resolution)?,
        })
    }

    /// A plank is a 3d object. Flatten it onto a plane.
    pub fn flatten(&self) -> Result<FlattenedPlank, Error> {
        let (first_len, triangles) = self.triangles()?;
        let mut top_line = vec![];
        let mut bottom_line = vec![];
        // Start with the leftmost points; assume WLOG they are at x=0.
        let mut top_pt = P2::new(0.0, 0.0);
        let mut bot_pt = P2::new(0.0, first_len);
        top_line.push(top_pt);
        bottom_line.push(bot_pt);
        // Add each triangle successively.
        for &(a, b, c, d) in &triangles {
            let new_top_pt = triangulate(top_pt, bot_pt, a, b);
            let new_bot_pt = triangulate(new_top_pt, bot_pt, c, d);
            top_line.push(new_top_pt);
            bottom_line.push(new_bot_pt);
            top_pt = new_top_pt;
            bot_pt = new_bot_pt;
        }
        Ok(FlattenedPlank {
            top_line: top_line,
            bottom_line: bottom_line,
        })
    }

    // Give the leftmost edge length, then triangle lengths from left to right.
    fn triangles(&self) -> Result<(f32, Vec<Triangles>), Error> {
        let top_pts = self.top_line.sample(Some(self.resolution));
        let bot_pts = self.bottom_line.sample(Some(self.resolution));
        let left_len = distance(&top_pts[0], &bot_pts[0]);
        let mut triangles = vec![];
        if top_pts.len() != bot_pts.len() {
            panic!(
                concat!(
                    "Plank unexpectedly has different number ",
                    "of top and bottom points. {} {}"
                ),
                top_pts.len(),
                bot_pts.len()
            );
        }
        let n = top_pts.len();
        for i in 0..n - 1 {
            triangles.push((
                distance(&top_pts[i], &top_pts[i + 1]),
                distance(&bot_pts[i], &top_pts[i + 1]),
                distance(&top_pts[i + 1], &bot_pts[i + 1]),
                distance(&bot_pts[i], &bot_pts[i + 1]),
            ));
        }
        Ok((left_len, triangles))
    }

    /// Render in 3d.
    pub fn render_3d(&self) -> Result<Tree, Error> {
        // Get the lines (bottom includes edges)
        let top_line = self.top_line.sample(None);
        let bottom_line =
            iter::once(top_line[0])
            .chain(self.bottom_line.sample(None).into_iter())
            .chain(iter::once(*top_line.last().unwrap()))
            .collect();
        // render the lines (top is dotted)
        let dots = ScadPath::new(top_line)
            .stroke(SCAD_STROKE)
            .link(PathStyle3::Dots)?;
        let solid = ScadPath::new(bottom_line)
            .stroke(SCAD_STROKE)
            .link(PathStyle3::Line)?;
        // return the rendering
        Ok(Tree::Union(vec![dots, solid]))
    }
}

type Triangles = (f32, f32, f32, f32);

/// Given two points and two edge lengths (and another number, for
/// horrifying edge cases), find a third point that makes a triangle
/// with those two points and those two edge lengths.
fn triangulate(pt1: P2, pt2: P2, x: f32, y: f32) -> P2 {
    // Use law of cosines.
    //    y*y = l*l + x*x -2lx*cos(pt1_angle)
    // -> pt1_angle = acos((l*l + x*x - y*y) / 2*l*x)
    let l = distance(&pt1, &pt2);
    if practically_zero(10.0 * l) {
        // There's no orientation information, so make some up.
        pt1 + V2::new(-x, 0.0)
    } else if practically_zero(x) {
        // Um, x is small and we would be dividing by it.
        // Use symmetry to divide by y instead.
        let pt2_angle =
            Rotation2::new(-f32::acos((l * l + y * y - x * x) / (2.0 * l * y)));
        pt2 + y * (pt2_angle * normalize(&(pt1 - pt2)))
    } else {
        let pt1_angle =
            Rotation2::new(f32::acos((l * l + x * x - y * y) / (2.0 * l * x)));
        pt1 + x * (pt1_angle * normalize(&(pt2 - pt1)))
    }
}

#[test]
fn test_triangulate() {
    let pt1 = P2::new(1.0, 4.0);
    let pt2 = P2::new(1.0, 1.0);
    let x = 4.0;
    let y = 5.0;
    assert_eq!(triangulate(pt1, pt2, x, y), P2::new(5.0, 4.0));
    let pt1 = P2::new(0.0, 1.0);
    let pt2 = P2::new(1.0, 0.0);
    let x = f32::sqrt(2.0);
    let y = f32::sqrt(2.0);
    assert_eq!(triangulate(pt1, pt2, x, y), P2::new(1.3660253, 1.3660254));
}
