use scad_dots::utils::{Axis, P2, P3};
use failure::Error;

use spec::{BreadthLine, HeightLine, Spec};
use spline::Spline;
use scad_dots::utils::distance;
use render_3d::{PathStyle3, ScadPath};
use render_2d::{PathStyle2, SvgColor, SvgPath};

use scad_dots::core::{MinMaxCoord, Tree};


// How near points must be to be considered equal, in 1/8th of an inch.
const EQUALITY_THRESHOLD: f32 = 8.0;

/// A ship's hull.
#[derive(MinMaxCoord)]
pub struct Hull {
    pub stations: Vec<Station>,
    #[min_max_coord(ignore)] pub heights: Vec<f32>,
    #[min_max_coord(ignore)] pub breadths: Vec<f32>,
    // TODO: store diagonals for drawing
}

/// A cross-section of the hull.
#[derive(MinMaxCoord)]
pub struct Station {
    pub points: Vec<P3>,
    #[min_max_coord(ignore)] pub position: f32,
    #[min_max_coord(ignore)] pub spline: Spline,
}

impl Station {
    pub fn render_3d(&self) -> Result<Tree, Error> {
        let path = ScadPath::new(self.points.clone())
            .stroke(10.0)
            .show_points()
            .link(PathStyle3::Line);
        path
    }
    pub fn render_spline_2d(&self) -> SvgPath {
        SvgPath::new(project(Axis::X, &self.spline.sample()))
            .stroke(SvgColor::Black, 2.5)
            .style(PathStyle2::LineWithDots)
    }

    pub fn render_points_2d(&self) -> SvgPath {
        SvgPath::new(project(Axis::X, &self.points))
            .stroke(SvgColor::Green, 2.0)
            .style(PathStyle2::Dots)
    }
}

impl Hull {
    pub fn draw_height_breadth_grid(&self) -> Vec<SvgPath> {
        let color = SvgColor::DarkGrey;
        let width = 2.;
        let style = PathStyle2::Line;

        let min_x = self.min_coord(Axis::Y);
        let max_x = self.max_coord(Axis::Y);
        let min_y = self.min_coord(Axis::Z);
        let max_y = self.max_coord(Axis::Z);

        let mut lines = Vec::new();
        for &height in &self.heights {
            let line = vec![P2::new(min_x, height), P2::new(max_x, height)];
            lines.push(
                SvgPath::new(reflect2(Axis::X, &line))
                    .stroke(color, width)
                    .style(style),
            );
            lines.push(SvgPath::new(line).stroke(color, width).style(style));
        }
        for &breadth in &self.breadths {
            let line = vec![P2::new(breadth, min_y), P2::new(breadth, max_y)];
            lines.push(
                SvgPath::new(reflect2(Axis::X, &line))
                    .stroke(color, width)
                    .style(style),
            );
            lines.push(SvgPath::new(line).stroke(color, width).style(style));
        }
        lines
    }

    pub fn draw_half_breadths(&self) -> Vec<SvgPath> {
        let mut paths = self.draw_height_breadth_grid();
        let half = (self.stations.len() as f32) / 2.;
        for (i, station) in self.stations.iter().enumerate() {
            let mut samples: Vec<P3> = station.spline.sample();
            let mut points: Vec<P3> = station.points.clone();
            if (i as f32) >= half {
                samples = reflect3(Axis::Y, &samples);
                points = reflect3(Axis::Y, &points);
            }
            paths.push(
                SvgPath::new(project(Axis::X, &samples))
                    .stroke(SvgColor::Black, 2.0)
                    .style(PathStyle2::Line),
            );
            paths.push(
                SvgPath::new(project(Axis::X, &points))
                    .stroke(SvgColor::Black, 2.0)
                    .style(PathStyle2::Dots),
            );
        }
        paths
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

        Ok(Hull {
            stations: stations,
            breadths: self.get_breadths(),
            heights: self.get_heights(),
        })
    }

    fn get_breadths(&self) -> Vec<f32> {
        let mut stored_breadths = vec![];
        for &(ref breadth, _) in &self.data.heights {
            match *breadth {
                BreadthLine::Sheer => (),
                BreadthLine::Wale => (),
                BreadthLine::ButOut(breadth) => stored_breadths.push(breadth as f32),
            }
        }
        stored_breadths
    }

    fn get_heights(&self) -> Vec<f32> {
        let mut stored_heights = vec![];
        for &(ref height, _) in &self.data.breadths {
            match *height {
                HeightLine::Sheer => (),
                HeightLine::WLUp(height) => stored_heights.push(height as f32),
            }
        }
        stored_heights
    }
}


fn sort_and_remove_duplicates(mut points: Vec<P3>) -> Vec<P3> {
    points.sort_by(|p, q| p.z.partial_cmp(&q.z).unwrap());
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

fn point(x: usize, y: usize, z: usize) -> P3 {
    P3::new(x as f32, y as f32, z as f32)
}

fn reflect2(axis: Axis, points: &[P2]) -> Vec<P2> {
    points
        .iter()
        .map(|p| {
            let mut new = *p;
            new[axis.index()] *= -1.;
            new
        })
        .collect()
}

fn reflect3(axis: Axis, points: &[P3]) -> Vec<P3> {
    points
        .iter()
        .map(|p| {
            let mut new = *p;
            new[axis.index()] *= -1.;
            new
        })
        .collect()
}

fn project(axis: Axis, points: &[P3]) -> Vec<P2> {
    let func = |p: &P3| match axis {
        Axis::X => P2::new(p.y, p.z),
        Axis::Y => P2::new(p.x, p.z),
        Axis::Z => P2::new(p.x, p.y),
    };
    points.iter().map(func).collect()
}
