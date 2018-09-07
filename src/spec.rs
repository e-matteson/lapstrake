//! Specifications for the ship hull.

use std::cmp;
use std::fmt;

use error::LapstrakeError;
use unit::*;

/// The spec for the hull of a ship, plus configuration options.
#[derive(Debug)]
pub struct Spec {
    pub data: Data,
    pub planks: Planks,
    pub config: Config,
}

/// A standard set of reference points for the hull shape.
#[derive(Debug)]
pub struct Data {
    /// The names of the stations (cross sections of the hull).
    pub stations: Vec<String>,
    /// The locations of each of the stations.
    pub positions: Vec<DataRow<HeightLine>>,
    /// For each station,
    /// the height above base
    /// at each half-breadth from center.
    pub heights: Vec<DataRow<BreadthLine>>,
    /// For each station,
    /// the half-breadth from centerline
    /// at each height above base.
    pub breadths: Vec<DataRow<HeightLine>>,
}

/// One row of Data. `T` is one of HeightLine, BreadthLine.
pub type DataRow<T> = (T, Vec<Option<Feet>>);

/// Where planks should lie on the hull.
#[derive(Debug, Clone)]
pub struct Planks {
    pub stations: Vec<PlankStation>,
    pub plank_locations: Vec<PlankRow>,
}

/// For a given plank, specifies where that plank should lie (as a
/// fraction between 0 for the bottom to 1 at the top) at each station
/// or fore-aft position.
pub type PlankRow = Vec<Option<f32>>;

/// A plank's location can be specified either along an existing
/// station, or along a cross-section of constant fore-aft position.
#[derive(Debug, Clone)]
pub enum PlankStation {
    Station(String),
    Position(Feet),
}

/// Configuration options.
#[derive(Debug, Deserialize)]
pub struct Config {
    pub resolution: usize,
}

/// A line along the hull of constant breadth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreadthLine {
    Sheer,
    Wale,
    ButOut(Feet),
}

/// A line along the hull of constant height.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeightLine {
    Sheer,
    WLUp(Feet),
}

impl Spec {
    /// Get the position of the nth station.
    /// (This is by index, not by name.)
    pub fn get_station_position(
        &self,
        station: usize,
        line: HeightLine,
    ) -> Result<Feet, LapstrakeError> {
        Spec::lookup(&self.data.positions, station, line)
    }

    /// Get the breadth of the sheer at the nth station.
    pub fn get_sheer_breadth(
        &self,
        station: usize,
    ) -> Result<Feet, LapstrakeError> {
        Spec::lookup(&self.data.breadths, station, HeightLine::Sheer)
    }

    /// Get the height of the sheer at the nth station.
    pub fn get_sheer_height(
        &self,
        station: usize,
    ) -> Result<Feet, LapstrakeError> {
        Spec::lookup(&self.data.heights, station, BreadthLine::Sheer)
    }

    fn lookup<M>(
        rows: &Vec<DataRow<M>>,
        station_index: usize,
        measurement: M,
    ) -> Result<Feet, LapstrakeError>
    where
        M: fmt::Debug + cmp::Eq + Copy,
    {
        for &(ref x, ref row) in rows.iter() {
            if *x == measurement {
                match row[station_index] {
                    Some(m) => return Ok(m),
                    None => (),
                }
            }
        }
        Err(LapstrakeError::Load(format!(
            "Could not find a measurement for {:?} at station index {}",
            measurement, station_index
        )))
    }
}
