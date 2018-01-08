use scad_dots::utils::{Axis, P2, P3, V2};
use scad_dots::core::{MinMaxCoord, Tree};
use failure::Error;

use util::{reflect2, reflect3, project_points};
use unit::Feet;
use render_3d::{PathStyle3, ScadPath, SCAD_STROKE};
use render_2d::{Bound, Bounded, PathStyle2, SvgCircle, SvgColor, SvgDoc,
                SvgGroup, SvgPath, SvgText, make_scale_bar};
use hull::{Hull, Station};



impl Hull {
    pub fn draw_half_breadths(&self) -> Vec<SvgPath> {
        let stroke = 0.02;
        let mut paths = self.draw_height_breadth_grid(stroke);
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

        let mut holes = SvgGroup::new();
        for pos in hole_positions {
            let hole = SvgCircle::new(pos, HOLE_DIAMETER / 2.)
                .stroke(SvgColor::Black, STROKE);
            if !intersection.contains(&hole.bound().unwrap()) {
                bail!("hole doesn't fit in overlap between cross-sections");
            }
            holes.append(hole)
        }

        let mut groups = Vec::new();
        for (name, mut path) in paths {
            // Add tab to each cross-section, for mounting it into a jig
            // let mut path = path.to_owned();
            let bound = path.bound().expect("path has no bound");
            let tab_length = V2::new(0.75 * bound.width(), 0.);
            let tab_center = P2::new(bound.center().x, 1.2 * max_y);
            path.append(vec![
                tab_center - tab_length / 2.,
                tab_center + tab_length / 2.,
            ]);

            // Add text label with name of cross-section
            let holes_bound = holes.bound().unwrap();
            let label = SvgText {
                lines: vec![name.into()],
                pos: holes_bound.center(),
                color: SvgColor::Black,
                size: 0.9 * (holes_bound.height() - 2. * HOLE_DIAMETER),
            };

            let mut group = SvgGroup::new();
            group.append(path);
            group.append(label);
            group.append(holes.clone());

            groups.push(group);
        }
        let mut doc = SvgDoc::new();
        let grid = SvgGroup::new_grid(groups, 1.1)?;
        let stack = SvgGroup::new_vertical(vec![make_scale_bar()?, grid], 1.1)?;
        doc.append(stack);
        Ok(doc)
    }

    pub fn draw_height_breadth_grid(&self, stroke: f32) -> Vec<SvgPath> {
        let color = SvgColor::DarkGrey;
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
                    .stroke(color, stroke)
                    .style(style),
            );
            lines.push(SvgPath::new(line).stroke(color, stroke).style(style));
        }
        for &breadth in &self.breadths {
            let line = vec![P2::new(breadth, min_y), P2::new(breadth, max_y)];
            lines.push(
                SvgPath::new(reflect2(Axis::X, &line))
                    .stroke(color, stroke)
                    .style(style),
            );
            lines.push(SvgPath::new(line).stroke(color, stroke).style(style));
        }
        lines
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

    fn get_cross_section_path(&self) -> SvgPath {
        // Draw right and left halves of cross-section
        let mut points: Vec<_> =
            self.spline.sample(None).into_iter().rev().collect();
        let left = reflect3(Axis::Y, &points);
        points.extend(left.iter().rev());
        SvgPath::new(project_points(Axis::X, &points))
            .stroke(SvgColor::Black, 0.02)
            .style(PathStyle2::Line)
            .close()
    }

    pub fn render_3d(&self) -> Result<Tree, Error> {
        let path = ScadPath::new(self.points.clone())
            .stroke(SCAD_STROKE)
            .show_points()
            .link(PathStyle3::Line);
        path
    }

}
