use scad_dots::core::MinMaxCoord;
use scad_dots::utils::{Axis, P2, P3};

use error::LapstrakeError;
use plank::{FlattenedPlank, Plank};
use spec::{BreadthLine, HeightLine, PlankStation, Planks, Spec};
use spline::Spline;
use unit::Feet;
use util::remove_duplicates;

/// A ship's hull.
#[derive(MinMaxCoord)]
pub struct Hull {
    pub stations: Vec<Station>,
    #[min_max_coord(ignore)]
    pub wale: Vec<P2>, // {x, z}
    #[min_max_coord(ignore)]
    pub heights: Vec<f32>,
    #[min_max_coord(ignore)]
    pub breadths: Vec<f32>,
    #[min_max_coord(ignore)]
    planks: Planks,
    #[min_max_coord(ignore)]
    resolution: usize,
}

/// A cross-section of the hull.
#[derive(MinMaxCoord)]
pub struct Station {
    #[min_max_coord(ignore)]
    pub name: String,
    pub points: Vec<P3>,
    #[min_max_coord(ignore)]
    pub spline: Spline,
}

impl Hull {
    /// Get a set of planks that can cover the hull.
    /// `n` is the number of planks for each side of the hull
    /// (so there will be 2n planks in total).
    /// `overlap` is how much each plank should overlap the next.
    /// Planks are meant to be layed out from the bottom of the ship
    /// to the top; as a result, the bottommost plank has no overlap.
    pub fn get_planks(&self) -> Result<Vec<Plank>, LapstrakeError> {
        let n = self.planks.plank_locations.len();
        let mut planks = vec![];
        for i in 0..n / 2 {
            let bot_line = self.get_plank_row(2 * i)?;
            let top_line = self.get_plank_row(2 * i + 1)?;
            planks.push(Plank::new(bot_line, top_line, self.resolution)?);
        }
        Ok(planks)
    }

    // (Used in get_planks)
    fn get_plank_row(&self, row: usize) -> Result<Vec<P3>, LapstrakeError> {
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
    pub fn get_flattened_planks(
        &self,
    ) -> Result<Vec<FlattenedPlank>, LapstrakeError> {
        FlattenedPlank::flatten_planks(self.get_planks()?)
    }

    /// Get a line across the hull that is a constant fraction `t`
    /// of the distance along the edge of each cross section.
    fn get_line(&self, t: f32) -> Result<Spline, LapstrakeError> {
        let points = self
            .stations
            .iter()
            .map(|station| station.at_t(t))
            .collect::<Result<_, LapstrakeError>>()?;
        Spline::new(points, self.resolution)
    }

    /// Get a station by name.
    fn get_station(
        &self,
        station_name: &str,
    ) -> Result<&Station, LapstrakeError> {
        for station in &self.stations {
            if station.name == station_name {
                return Ok(station);
            }
        }
        Err(LapstrakeError::General(format!(
            "Station {} not found.",
            station_name,
        )))
    }

    // Get a point a fraction `t` of the way along the curve of the
    // given station.
    fn get_point(
        &self,
        t: f32,
        station: &PlankStation,
    ) -> Result<P3, LapstrakeError> {
        match station {
            &PlankStation::Station(ref station_name) => {
                Ok(self.get_station(station_name)?.at_t(t)?)
            }
            &PlankStation::Position(posn) => {
                Ok(self.hallucinate_station(posn)?.at_t(t)?)
            }
        }
    }

    /// Construct a station at the given fore-aft position.
    pub fn hallucinate_station(
        &self,
        posn: Feet,
    ) -> Result<Station, LapstrakeError> {
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
}

impl Station {
    pub fn new(
        name: String,
        points: Vec<P3>,
        resolution: usize,
    ) -> Result<Station, LapstrakeError> {
        Ok(Station {
            name: name,
            points: points.clone(),
            spline: Spline::new(points, resolution)?,
        })
    }

    /// Get a point along the curve of this station a fraction `t` of
    /// the way along the curve.
    pub fn at_t(&self, t: f32) -> Result<P3, LapstrakeError> {
        self.spline.at_t(t)
    }
}

impl Spec {
    pub fn get_hull(&self) -> Result<Hull, LapstrakeError> {
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
                    let posn =
                        self.get_station_position(i, HeightLine::Sheer)?;
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
