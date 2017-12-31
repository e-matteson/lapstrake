use std::cmp::Ordering;
use scad_dots::utils::{Axis, P2, P3};
use failure::Error;

use spec::{BreadthLine, DiagonalLine, HeightLine, Spec};
use spline::Spline;
use scad_dots::harness::preview_model;
use scad_dots::utils::distance;
use render_3d::{PathStyle, ScadPath};
use render_2d::{SvgColor, SvgPath};
use svg::node::element::Group;

use scad_dots::core::{chain, mark, Dot, DotAlign, DotSpec, Shape, Tree};


// How near points must be to be considered equal, in 1/8th of an inch.
const EQUALITY_THRESHOLD: f32 = 8.0;

/// A ship's hull.
pub struct Hull {
    pub stations: Vec<Station>
}

/// A cross-section of the hull.
pub struct Station {
    pub position: f32,
    pub points: Vec<P3>,
    pub spline: Spline,
}

impl Station {
    pub fn render_3d(&self) -> Result<Tree, Error> {
        let path = ScadPath::new(self.points.clone())
            .stroke(10.0)
            .show_points()
            .link(PathStyle::Line);
        path
    }

    pub fn render_spline_2d(&self, out: &str) {
        let points = &self.spline.sample();
        let path = SvgPath::new(project(Axis::X, points))
            .stroke(SvgColor::Black, 2.0)
            .show_points()
            .save(out);
    }

    pub fn render_points_2d(&self, out: &str) {
        let points = &self.points;
        let path = SvgPath::new(project(Axis::X, points))
            .stroke(SvgColor::Black, 2.0)
            .show_points()
            .save(out);
    }
}

impl Spec {
    pub fn get_hull(&self, resolution: usize) -> Result<Hull, Error> {
        let data = &self.data;
        let mut stations = vec![];
        for (i, &position) in data.positions.iter().enumerate() {
            let mut points = vec![];
            // Add the sheer point.
            let sheer_breadth = self.get_sheer_breadth(i)?;
            let sheer_height = self.get_sheer_height(i)?;
            points.push(point(position, sheer_breadth, sheer_height));
            // Add all other points.
            for &(ref breadth, ref row) in &data.heights {
                match *breadth {
                    BreadthLine::Sheer => (),
                    BreadthLine::Wale => (),
                    BreadthLine::ButOut(breadth) => if let Some(height) = row[i] {
                        points.push(point(position, breadth, height));
                    },
                }
            }
            for &(ref height, ref row) in &data.breadths {
                match *height {
                    HeightLine::Sheer => (),
                    HeightLine::WLUp(height) => if let Some(breadth) = row[i] {
                        points.push(point(position, breadth, height));
                    },
                }
            }
            // TODO: diagonals
            // The points are out of order, and may contain duplicates.
            // Sort them and remove the duplicates.
            let points = sort_and_remove_duplicates(points);
            // Construct the station (cross section).
            let station = Station {
                position: position as f32,
                points: points.clone(),
                spline: Spline::new(points, resolution)?,
            };
            stations.push(station);
        }
        Ok(Hull{
            stations: stations
        })
    }
}

fn sort_and_remove_duplicates(mut points: Vec<P3>) -> Vec<P3> {
    points.sort_by(|p, q| p.z.partial_cmp(&q.z).unwrap());
    let mut good_points = vec!();
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

fn point(x: usize, y: usize, z: usize) -> P3 {
    P3::new(x as f32, y as f32, z as f32)
}

fn reflect_y(points: &[P3]) -> Vec<P3> {
    points.iter().map(|p| P3::new(p.x, -p.y, p.z)).collect()
}

fn project(axis: Axis, points: &[P3]) -> Vec<P2> {
    let func = |p: &P3| match axis {
        Axis::X => P2::new(p.y, p.z),
        Axis::Y => P2::new(p.x, p.z),
        Axis::Z => P2::new(p.x, p.y),
    };
    points.iter().map(func).collect()
}
