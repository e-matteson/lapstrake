//! Specs for the ship hull.

use std::fmt;
use std::cmp;
use std::str::FromStr;
use failure::{Error, ResultExt};

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
    /// For each station,
    /// the distance along the diagonal lines (given in Config).
    pub diagonals: Vec<DataRow<DiagonalLine>>,
}

/// One row of Data. `T` is one of HeightLine, BreadthLine, DiagonalLine.
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
    pub station_resolution: usize,
    pub plank_resolution: usize,
    pub number_of_planks: usize,
    plank_overlap: String,
}

impl Config {
    pub fn plank_overlap(&self) -> Result<f32, Error> {
        Ok(Feet::parse(&self.plank_overlap)?.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreadthLine {
    Sheer,
    Wale,
    ButOut(Feet),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeightLine {
    Sheer,
    WLUp(Feet),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagonalLine {
    A,
    B,
}

impl Spec {
    pub fn get_station_position(
        &self,
        station: usize,
        line: HeightLine,
    ) -> Result<Feet, Error> {
        Spec::lookup(&self.data.positions, station, line)
    }

    pub fn get_sheer_breadth(&self, station: usize) -> Result<Feet, Error> {
        Spec::lookup(&self.data.breadths, station, HeightLine::Sheer)
    }

    pub fn get_sheer_height(&self, station: usize) -> Result<Feet, Error> {
        Spec::lookup(&self.data.heights, station, BreadthLine::Sheer)
    }

    fn lookup<M>(
        rows: &Vec<DataRow<M>>,
        station_index: usize,
        measurement: M,
    ) -> Result<Feet, Error>
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
        bail!(
            "Could not find a measurement for {:?} at station index {}",
            measurement,
            station_index
        );
    }
}

impl FromStr for BreadthLine {
    type Err = Error;
    fn from_str(text: &str) -> Result<BreadthLine, Error> {
        match text.to_lowercase().as_str() {
            "sheer" => Ok(BreadthLine::Sheer),
            "wale" => Ok(BreadthLine::Wale),
            text => {
                let feet =
                    Feet::parse(text).context("Was unable to read height.")?;
                Ok(BreadthLine::ButOut(feet.into()))
            }
        }
    }
}

impl FromStr for HeightLine {
    type Err = Error;
    fn from_str(text: &str) -> Result<HeightLine, Error> {
        match text.to_lowercase().as_str() {
            "sheer" => Ok(HeightLine::Sheer),
            text => {
                let feet =
                    Feet::parse(text).context("Was unable to read breadth.")?;
                Ok(HeightLine::WLUp(feet.into()))
            }
        }
    }
}

impl FromStr for DiagonalLine {
    type Err = Error;
    fn from_str(text: &str) -> Result<DiagonalLine, Error> {
        match text.to_lowercase().as_str() {
            "a" => Ok(DiagonalLine::A),
            "b" => Ok(DiagonalLine::B),
            _ => bail!(
                concat!("Could not read diagonal {}. Expected A or B."),
                text
            ),
        }
    }
}
