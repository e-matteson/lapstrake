use scad_dots::utils::{Axis, P2, P3};
use failure::Error;

use spec::{BreadthLine, HeightLine, Spec};
use spline::Spline;
use nalgebra::{normalize, Rotation2};
use scad_dots::utils::{distance, rotation_between};
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

#[derive(Debug, Clone)]
pub struct Plank {
    pub top_line: Spline,
    pub bottom_line: Spline,
}

#[derive(Debug, Clone)]
pub struct FlattenedPlank {
    pub top_line: Vec<P2>,
    pub bottom_line: Vec<P2>,
}

impl FlattenedPlank {
    /// Render as an SVG path.
    pub fn render_2d(&self) -> SvgPath {
        SvgPath::new(self.get_path())
            .stroke(SvgColor::Black, 2.0)
            .style(PathStyle2::Line)
    }

    fn get_path(&self) -> Vec<P2> {
        let top_line = self.top_line.clone();
        let mut bottom_line = self.bottom_line.clone();
        bottom_line.reverse();

        let mut points = vec![];
        points.extend(top_line);
        points.extend(bottom_line);
        points.push(self.top_line[0]);
        points
    }
}

impl Plank {
    /// A plank is a 3d object. Flatten it out to fit on a piece of paper.
    pub fn flatten(&self) -> Result<FlattenedPlank, Error> {
        let (first_len, quads) = self.quads()?;
        let mut top_line = vec![];
        let mut bottom_line = vec![];
        // Start with the leftmost points; assume WLOG they are at x=0.
        let mut top_pt = P2::new(0.0, 0.0);
        let mut bot_pt = P2::new(0.0, first_len);
        top_line.push(top_pt);
        bottom_line.push(bot_pt);
        // Add each quad successively.
        for quad in &quads {
            let top_vec = normalize(&(bot_pt - top_pt)) * quad.top_len;
            let top_rot = Rotation2::new(-quad.top_angle);
            let new_top_pt = top_pt + top_rot * top_vec;
            let bot_vec = normalize(&(top_pt - bot_pt)) * quad.bot_len;
            let bot_rot = Rotation2::new(quad.bot_angle);
            let new_bot_pt = bot_pt + bot_rot * bot_vec;
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

    // Give the "leftmost" edge length, then quads from "left" to "right".
    fn quads(&self) -> Result<(f32, Vec<Quad>), Error> {
        let top_pts = self.top_line.sample();
        let bot_pts = self.bottom_line.sample();
        let left_len = distance(&top_pts[0], &bot_pts[0]);
        let mut quads = vec![];
        if top_pts.len() != bot_pts.len() {
            panic!(concat!(
                "Plank unexpectedly has different number ",
                "of top and bottom points."
            ));
        }
        let n = top_pts.len();
        for i in 0..n - 1 {
            quads.push(Quad {
                top_len: distance(&top_pts[i], &top_pts[i + 1]),
                bot_len: distance(&bot_pts[i], &bot_pts[i + 1]),
                top_angle: rotation_between(
                    &(top_pts[i + 1] - top_pts[i]),
                    &(top_pts[i] - bot_pts[i]),
                )?.angle(),
                bot_angle: rotation_between(
                    &(bot_pts[i + 1] - bot_pts[i]),
                    &(bot_pts[i] - top_pts[i]),
                )?.angle(),
            });
        }
        Ok((left_len, quads))
    }
}

struct Quad {
    // top left interior angle
    top_angle: f32,
    // bottom left interior angle
    bot_angle: f32,
    // top edge length
    top_len: f32,
    // bottom edge length
    bot_len: f32,
}

impl Hull {
    /// Get a set of planks that can cover the hull.
    /// `n` is the number of planks for each side of the hull
    /// (so there will be 2n planks in total).
    /// `overlap` is how much each plank should overlap the next.
    /// Planks are meant to be layed out from the bottom of the ship
    /// to the top; as a result, the bottommost plank has no overlap.
    pub fn get_planks(
        &self,
        n: usize,
        overlap: usize,
        resolution: usize,
    ) -> Result<Vec<Plank>, Error> {
        let mut planks = vec![];
        for i in 0..n {
            let f_bottom = i as f32 / n as f32;
            let f_top = (i + 1) as f32 / n as f32;
            let at_end = i + 1 == n;
            let offset = if at_end { 0 } else { overlap };
            let bottom_line = self.get_line(f_bottom, 0);
            let top_line = self.get_line(f_top, offset);
            planks.push(Plank {
                bottom_line: Spline::new(bottom_line, resolution)?,
                top_line: Spline::new(top_line, resolution)?,
            });
        }
        Ok(planks)
    }

    /// Get a line across the hull that is a constant fraction `f`
    /// of the distance along the edge of each cross section.
    fn get_line(&self, f: f32, offset: usize) -> Vec<P3> {
        self.stations
            .iter()
            .map(|station| {
                let len = station.spline.length();
                let dist = f * len + offset as f32;
                station.spline.at(dist)
            })
            .collect()
    }

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
            .stroke(SvgColor::Black, 2.0)
            .style(PathStyle2::Line)
    }

    pub fn render_points_2d(&self) -> SvgPath {
        SvgPath::new(project(Axis::X, &self.points))
            .stroke(SvgColor::Green, 2.0)
            .style(PathStyle2::Dots)
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
                    BreadthLine::ButOut(breadth) => {
                        if let Some(height) = row[i] {
                            points.push(point(position, breadth, height));
                        }
                    }
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
                BreadthLine::ButOut(breadth) => {
                    stored_breadths.push(breadth as f32)
                }
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
