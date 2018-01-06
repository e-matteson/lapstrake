use nalgebra::{normalize, Rotation2};
use scad_dots::utils::{Axis, P2, P3, V2};
use scad_dots::core::{MinMaxCoord, Tree};
use failure::Error;

use util::{practically_zero, remove_duplicates, reflect2, reflect3};
use util::EQUALITY_THRESHOLD;
use unit::Feet;
use spec::{BreadthLine, HeightLine, PlankStation, Planks, Spec};
use spline::Spline;
use scad_dots::utils::distance;
use render_3d::{PathStyle3, ScadPath, SCAD_STROKE};
use render_2d::{Bound, PathStyle2, SvgCircle, SvgColor, SvgDoc, SvgGroup,
                SvgPath};
use util::project_points;
// use svg::Node;
// use svg::node::element::Group;

/// A ship's hull.
#[derive(MinMaxCoord)]
pub struct Hull {
    pub stations: Vec<Station>,
    #[min_max_coord(ignore)] pub wale: Vec<P2>, // {x, z}
    #[min_max_coord(ignore)] pub heights: Vec<f32>,
    #[min_max_coord(ignore)] pub breadths: Vec<f32>,
    #[min_max_coord(ignore)] planks: Planks,
    #[min_max_coord(ignore)] resolution: usize,
}

/// A cross-section of the hull.
#[derive(MinMaxCoord)]
pub struct Station {
    #[min_max_coord(ignore)] name: String,
    pub points: Vec<P3>,
    #[min_max_coord(ignore)] pub spline: Spline,
}

#[derive(Debug, Clone)]
pub struct Plank {
    pub top_line: Spline,
    pub bottom_line: Spline,
    pub resolution: usize,
}

#[derive(Debug, Clone, MinMaxCoord)]
pub struct FlattenedPlank {
    pub top_line: Vec<P2>,
    pub bottom_line: Vec<P2>,
}

impl FlattenedPlank {
    /// Render as an SVG path.
    pub fn render_2d(&self) -> SvgPath {
        SvgPath::new(self.get_path())
            .stroke(SvgColor::Black, 0.01)
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
}

impl Plank {
    fn new(
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

    /// A plank is a 3d object. Flatten it out to fit on a piece of paper.
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

    pub fn render_3d(&self) -> Result<Tree, Error> {
        // Get the lines (bottom includes edges)
        let top_line = self.top_line.sample(None);
        let mut bottom_line = vec![];
        bottom_line.push(top_line[0]);
        bottom_line.extend(self.bottom_line.sample(None));
        bottom_line.push(*top_line.last().unwrap());
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

impl Hull {
    /// Get a set of planks that can cover the hull.
    /// `n` is the number of planks for each side of the hull
    /// (so there will be 2n planks in total).
    /// `overlap` is how much each plank should overlap the next.
    /// Planks are meant to be layed out from the bottom of the ship
    /// to the top; as a result, the bottommost plank has no overlap.
    pub fn get_planks(&self) -> Result<Vec<Plank>, Error> {
        let n = self.planks.plank_locations.len() / 2;
        let mut planks = vec![];
        for i in 0..n {
            let i = i * 2;
            let bot_locs = &self.planks.plank_locations[i];
            let top_locs = &self.planks.plank_locations[i + 1];
            let mut bot_line = vec![];
            let mut top_line = vec![];
            for (j, ref station) in self.planks.stations.iter().enumerate() {
                let bot_f = bot_locs[j];
                let top_f = top_locs[j];

                if let Some(bot_f) = bot_f {
                    bot_line.push(self.get_point(bot_f, station)?);
                }
                if let Some(top_f) = top_f {
                    top_line.push(self.get_point(top_f, station)?);
                }
            }
            planks.push(Plank::new(bot_line, top_line, self.resolution)?)
        }
        Ok(planks)
    }

    /// Get planks flattened to 2d.
    pub fn get_flattened_planks(&self) -> Result<Vec<FlattenedPlank>, Error> {
        let flattened_planks = self.get_planks()?
            .into_iter()
            .map(|plank| plank.flatten())
            .collect::<Result<Vec<FlattenedPlank>, Error>>()?;
        let flattened_planks = flattened_planks.into_iter();
        let mut layed_planks = vec![];
        let mut last_y = None;
        for mut plank in flattened_planks {
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

    /// Get a line across the hull that is a constant fraction `t`
    /// of the distance along the edge of each cross section.
    fn get_line(&self, t: f32) -> Result<Spline, Error> {
        let points = self.stations
            .iter()
            .map(|station| station.at_t(t))
            .collect();
        Spline::new(points, self.resolution)
    }

    // Get a position on the station that is a constant fraction `f`
    // of the distance along the edge of the station.
    fn get_station(&self, station_name: &str) -> Result<&Station, Error> {
        for station in &self.stations {
            if station.name == station_name {
                return Ok(station);
            }
        }
        bail!("Station {} not found.", station_name);
    }

    // Get a point a fraction `t` of the way along the curve of the
    // given station.
    fn get_point(&self, t: f32, station: &PlankStation) -> Result<P3, Error> {
        match station {
            &PlankStation::Station(ref station_name) => {
                Ok(self.get_station(station_name)?.at_t(t))
            }
            &PlankStation::Position(posn) => {
                Ok(self.hallucinate_station(posn)?.at_t(t))
            }
        }
    }

    pub fn hallucinate_station(&self, posn: Feet) -> Result<Station, Error> {
        let mut points = vec![];
        let resolution = 10;
        for i in 0..resolution + 1 {
            let t = i as f32 / resolution as f32;
            let line = self.get_line(t)?;
            points.push(line.at_x(posn.into())?);
        }
        let name = format!("{}", posn);
        Station::new(name, points, self.resolution)
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
        let stroke = 0.02;
        let mut paths = self.draw_height_breadth_grid();
        let half = (self.stations.len() as f32) / 2.;
        for (i, station) in self.stations.iter().enumerate() {
            let mut samples: Vec<P3> = station.spline.sample(None);
            let mut points: Vec<P3> = station.points.clone();
            if (i as f32) >= half {
                samples = reflect3(Axis::Y, &samples);
                points = reflect3(Axis::Y, &points);
            }
            paths.push(
                SvgPath::new(project_points(Axis::X, &samples))
                    .stroke(SvgColor::Black, stroke)
                    .style(PathStyle2::Line),
            );
            paths.push(
                SvgPath::new(project_points(Axis::X, &points))
                    .stroke(SvgColor::Black, stroke)
                    .style(PathStyle2::Dots),
            );
        }
        paths
    }

    pub fn draw_cross_sections(
        &self,
        excluded: &[String],
    ) -> Result<SvgDoc, Error> {
        const HOLE_DIAMETER: f32 = 0.125;
        const STROKE: f32 = 0.02;
        let mut paths = Vec::new();
        let mut bounds = Vec::new();
        for station in &self.stations {
            if excluded.contains(&station.name) {
                continue;
            }
            let path = station
                .get_cross_section_path()
                .stroke(SvgColor::Black, 0.02);
            bounds.push(path.bound());
            paths.push((station.name.clone(), path));
        }

        let max_y = Bound::union_all(&bounds).high.y;
        let intersection = Bound::intersect_all(&bounds).ok_or_else(|| {
            format_err!(
                "cross-sections have no overlap in which to place alignment holes"
            )
        })?;

        let hole_positions = vec![
            intersection.relative_pos(0.5, 0.33),
            intersection.relative_pos(0.5, 0.66),
        ];

        let make_holes = || -> Result<SvgGroup, Error> {
            // This is to work around the fact that SvgGroup cannot be cloned,
            // because svg library types can't be. Recreate the group everytime,
            // instead.
            let mut holes = SvgGroup::new();
            for &pos in &hole_positions {
                let hole = SvgCircle::new(pos, HOLE_DIAMETER/2.)
                    .stroke(SvgColor::Black, STROKE);
                if !intersection.contains(&hole.bound()) {
                    bail!("hole doesn't fit in overlap between cross-sections");
                }
                holes.append_node(
                    hole.finalize()
                )
            }
            Ok(holes)
        };

        let mut doc = SvgDoc::new();
        let cols = (paths.len() as f32).sqrt() as usize;

        let mut col_bound = Bound::new();
        for col in paths.chunks(cols) {
            for &(ref _name, ref path) in col {
                // Add tab to each cross-section, for mounting it into a jig
                let mut path = path.to_owned();
                let tab_length = V2::new(0.75 * path.bound().width(), 0.);
                let tab_center = P2::new(path.bound().center().x, 1.2 * max_y);
                path.append(vec![tab_center - tab_length /2., tab_center + tab_length /2.]);

                // // TODO Add text label with name of cross-section
                // let label = SvgText {
                //     lines: vec![name.into()],
                //     pos: path.bound().center(),
                //     color: SvgColor::Black,
                //     size: 0.2,
                // }

                let mut group = SvgGroup::new();
                group.append_path(path.to_owned());

                // Translate into grid
                group.translate_to(col_bound.relative_pos(0., 1.1));
                let holes = make_holes()?;
                group.append_group(holes);
                // TODO letters
                col_bound = col_bound.union(group.bound());
                doc.append_group(group);
            }
            col_bound = Bound::empty_at(col_bound.relative_pos(1.1, 0.));
        }
        Ok(doc)
    }

    pub fn render_station_at(&self, posn: Feet) -> Result<Tree, Error> {
        let station = self.hallucinate_station(posn)?;
        Ok(ScadPath::new(station.points.clone())
            .stroke(0.1)
            .show_points()
            .link(PathStyle3::Line)?)
    }

    pub fn render_stations(&self) -> Result<Tree, Error> {
        let mut trees = Vec::new();
        for station in &self.stations {
            trees.push(ScadPath::new(station.points.clone())
                .stroke(0.1)
                .show_points()
                .link(PathStyle3::Line)?)
        }
        Ok(Tree::Union(trees))
    }
}

impl Station {
    pub fn new(
        name: String,
        points: Vec<P3>,
        resolution: usize,
    ) -> Result<Station, Error> {
        Ok(Station {
            name: name,
            points: points.clone(),
            spline: Spline::new(points, resolution)?,
        })
    }

    pub fn render_3d(&self) -> Result<Tree, Error> {
        let path = ScadPath::new(self.points.clone())
            .stroke(SCAD_STROKE)
            .show_points()
            .link(PathStyle3::Line);
        path
    }

    pub fn render_spline_2d(&self) -> SvgPath {
        SvgPath::new(project_points(Axis::X, &self.spline.sample(None)))
            .stroke(SvgColor::Black, 2.0)
            .style(PathStyle2::Line)
    }

    pub fn render_points_2d(&self) -> SvgPath {
        SvgPath::new(project_points(Axis::X, &self.points))
            .stroke(SvgColor::Green, 2.0)
            .style(PathStyle2::Dots)
    }

    /// Get a point along the curve of this station a fraction `t` of
    /// the way along the curve.
    pub fn at_t(&self, t: f32) -> P3 {
        self.spline.at_t(t)
    }

    fn get_cross_section_path(&self) -> SvgPath {
        // Draw right and left halves of cross-section
        let mut points: Vec<_> = self.spline.sample(None)
            .into_iter().rev().collect();
        let left = reflect3(Axis::Y, &points);
        points.extend(left.iter().rev());
        SvgPath::new(project_points(Axis::X, &points))
            .stroke(SvgColor::Black, 0.02)
            .style(PathStyle2::Line)
            .close()
    }
}

impl Spec {
    pub fn get_hull(&self) -> Result<Hull, Error> {
        let data = &self.data;
        let resolution = self.config.resolution;
        let mut stations = vec![];
        let mut wale = vec![];
        for i in 0..data.stations.len() {
            let mut points = vec![];
            // Add the sheer point.
            let sheer_breadth = self.get_sheer_breadth(i)?;
            let sheer_height = self.get_sheer_height(i)?;
            let sheer_posn = self.get_station_position(i, HeightLine::Sheer)?;
            points.push(point(sheer_posn, sheer_breadth, sheer_height));
            // Add the height measurements. Assume they are at the
            // positions given by the sheer for that station.
            for &(ref breadth, ref row) in &data.heights {
                if let Some(height) = row[i] {
                    let posn = self.get_station_position(i, HeightLine::Sheer)?;
                    match *breadth {
                        BreadthLine::Sheer => (),
                        BreadthLine::Wale => {
                            wale.push(P2::new(posn.into(), height.into()));
                        }
                        BreadthLine::ButOut(breadth) => {
                            points.push(point(posn, breadth, height));
                        }
                    }
                }
            }
            // Add the breadth measurements.
            for &(ref height, ref row) in &data.breadths {
                if let Some(breadth) = row[i] {
                    let posn = self.get_station_position(i, *height)?;
                    match *height {
                        HeightLine::Sheer => (),
                        HeightLine::WLUp(height) => {
                            points.push(point(posn, breadth, height));
                        }
                    }
                }
            }
            // The points are out of order, and may contain duplicates.
            // Sort them and remove the duplicates.
            points.sort_by(|p, q| p.z.partial_cmp(&q.z).unwrap());
            let points = remove_duplicates(points);
            // Construct the station (cross section).
            stations.push(Station::new(
                data.stations[i].to_string(),
                points,
                resolution,
            )?);
        }

        Ok(Hull {
            stations: stations,
            breadths: self.get_breadths(),
            heights: self.get_heights(),
            wale: wale,
            planks: self.planks.clone(),
            resolution: self.config.resolution,
        })
    }

    fn get_breadths(&self) -> Vec<f32> {
        let mut stored_breadths = vec![];
        for &(ref breadth, _) in &self.data.heights {
            match *breadth {
                BreadthLine::Sheer => (),
                BreadthLine::Wale => (),
                BreadthLine::ButOut(breadth) => {
                    stored_breadths.push(breadth.into())
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
                HeightLine::WLUp(height) => stored_heights.push(height.into()),
            }
        }
        stored_heights
    }
}

fn point(x: Feet, y: Feet, z: Feet) -> P3 {
    P3::new(x.into(), y.into(), z.into())
}

/// Given two points and two edge lengths (and another number, for
/// horrifying edge cases), find a third point that makes a triangle
/// with those two points and those two edge lengths.
fn triangulate(pt1: P2, pt2: P2, x: f32, y: f32) -> P2 {
    // Use law of cosines.
    //    yy = ll + xx -2lx*cos(pt1_angle)
    // -> pt1_angle = acos((ll + xx - yy) / 2lx)
    let l = distance(&pt1, &pt2);
    if practically_zero(10.0 * l) {
        pt1 + V2::new(-x, 0.0)
    } else if practically_zero(x) {
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
