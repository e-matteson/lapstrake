use scad_dots::utils::{Axis, P2, P3, V2};
use scad_dots::core::{MinMaxCoord, Tree};
use failure::Error;

use util::{remove_duplicates, reflect2, reflect3, project_points};
use unit::Feet;
use spec::{BreadthLine, HeightLine, PlankStation, Planks, Spec};
use spline::Spline;
use plank::{Plank, FlattenedPlank};
use render_3d::{PathStyle3, ScadPath, SCAD_STROKE};
use render_2d::{Bound, PathStyle2, SvgCircle, SvgColor, SvgDoc, SvgGroup,
                SvgPath};
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

impl Hull {
    /// Get a set of planks that can cover the hull.
    /// `n` is the number of planks for each side of the hull
    /// (so there will be 2n planks in total).
    /// `overlap` is how much each plank should overlap the next.
    /// Planks are meant to be layed out from the bottom of the ship
    /// to the top; as a result, the bottommost plank has no overlap.
    pub fn get_planks(&self) -> Result<Vec<Plank>, Error> {
        let n = self.planks.plank_locations.len();
        let mut planks = vec![];
        for i in 0..n/2 {
            let bot_line = self.get_plank_row(2*i)?;
            let top_line = self.get_plank_row(2*i + 1)?;
            planks.push(Plank::new(bot_line, top_line, self.resolution)?);
        }
        Ok(planks)
    }

    // (Used in get_planks)
    fn get_plank_row(&self, row: usize) -> Result<Vec<P3>, Error> {
        let locs = &self.planks.plank_locations[row];
        let mut line = vec![];
        for (i, ref station) in self.planks.stations.iter().enumerate() {
            if let Some(f) = locs[i] {
                line.push(self.get_point(f, station)?);
            }
        }
        Ok(line)
    }

    /// Get planks flattened to 2d. Place them nicely, without overlap.
    pub fn get_flattened_planks(&self) -> Result<Vec<FlattenedPlank>, Error> {
        FlattenedPlank::flatten_planks(self.get_planks()?)
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

    /// Get a station by name.
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

    /// Construct a station at the given fore-aft position.
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

    /// Construct a station at the given for-aft position, then render it.
    pub fn render_station_at(&self, posn: Feet) -> Result<Tree, Error> {
        let station = self.hallucinate_station(posn)?;
        Ok(ScadPath::new(station.points.clone())
            .stroke(0.1)
            .show_points()
            .link(PathStyle3::Line)?)
    }

    /// Render all of this hull's stations.
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
