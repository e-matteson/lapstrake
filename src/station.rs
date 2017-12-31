use std::cmp::Ordering;
use scad_dots::utils::{Axis, P2, P3};
use failure::Error;

use spec::{BreadthLine, DiagonalLine, HeightLine, Spec};
use spline::Spline;
use scad_dots::harness::preview_model;
use render_3d::{PathStyle, ScadPath};
use render_2d::{SvgColor, SvgPath};
use svg::node::element::Group;

use scad_dots::core::{chain, mark, Dot, DotAlign, DotSpec, Shape, Tree};

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

    // pub fn render_2d(&self) -> Result<Group, Error> {
    pub fn render_2d(&self) {
        let path = SvgPath::new(project(Axis::X, &self.points))
            .stroke(SvgColor::Black, 2.0)
            .show_points()
            .save("out.svg");
    }
}

impl Spec {
    pub fn get_stations(&self, resolution: usize) -> Result<Vec<Station>, Error> {
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
            // The points came in out of order: sort them.
            points.sort_by(|p, q| p.z.partial_cmp(&q.z).unwrap());
            // Construct the station (cross section).
            let station = Station {
                position: position as f32,
                points: points.clone(),
                spline: Spline::new(points, resolution)?,
            };
            stations.push(station);
        }
        Ok(stations)
    }
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
