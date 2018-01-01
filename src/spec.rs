//! Specs for the ship hull.

use std::str::FromStr;
use failure::{Error, ResultExt};

use unit::*;

/// The spec for the hull of a ship, plus configuration options.
#[derive(Debug)]
pub struct Spec {
    pub config: Config,
    pub data: Data,
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
    pub fn plank_overlap(&self) -> Result<usize, Error> {
        Ok(Feet::parse(&self.plank_overlap)?.into())
    }
}

/// A standard set of reference points for the hull shape.
#[derive(Debug)]
pub struct Data {
    /// The names of the stations (cross sections of the hull).
    pub stations: Vec<String>,
    /// The locations of each of the stations.
    pub positions: Vec<usize>,
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
pub type DataRow<T> = (T, Vec<Option<usize>>);

#[derive(Debug, Clone, Copy)]
pub enum BreadthLine {
    Sheer,
    Wale,
    ButOut(usize),
}

#[derive(Debug, Clone, Copy)]
pub enum HeightLine {
    Sheer,
    WLUp(usize),
}

#[derive(Debug, Clone, Copy)]
pub enum DiagonalLine {
    A,
    B,
}

impl Spec {
    pub fn get_sheer_breadth(
        &self,
        station_index: usize,
    ) -> Result<usize, Error> {
        for &(ref height, ref row) in &self.data.breadths {
            match *height {
                HeightLine::Sheer => match row[station_index] {
                    Some(x) => return Ok(x),
                    None => bail!("Sheer is required, but was omitted."),
                },
                _ => (),
            }
        }
        bail!("Did not find sheer breadth.")
    }

    pub fn get_sheer_height(
        &self,
        station_index: usize,
    ) -> Result<usize, Error> {
        for &(ref breadth, ref row) in &self.data.heights {
            match *breadth {
                BreadthLine::Sheer => match row[station_index] {
                    Some(x) => return Ok(x),
                    None => bail!("Sheer is required, but was omitted."),
                },
                _ => (),
            }
        }
        bail!("Did not find sheer height.")
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
