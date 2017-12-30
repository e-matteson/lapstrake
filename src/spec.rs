use std::str::FromStr;
use failure::{Error, ResultExt};

use unit::*;


pub type DataRow<T> = (T, Vec<Option<usize>>);

#[derive(Debug)]
pub struct Spec {
    pub config: Config,
    pub data:   Data
}

#[derive(Debug)]
pub struct Config {
    pub sheer_thickness: usize,
    pub wale_thickness: usize
}

#[derive(Debug)]
pub struct Data {
    pub stations:  Vec<String>,
    pub positions: Vec<usize>,
    pub heights:   Vec<DataRow<Height>>,
    pub breadths:  Vec<DataRow<Breadth>>,
    pub diagonals: Vec<DataRow<Diagonal>>
}

#[derive(Debug)]
pub enum Height {
    Sheer,
    Wale,
    Rabbet,
    Height(usize)
}

#[derive(Debug)]
pub enum Breadth {
    Sheer,
    Breadth(usize)
}

#[derive(Debug)]
pub enum Diagonal {
    A,
    B
}

impl FromStr for Height {
    type Err = Error;
    fn from_str(text: &str) -> Result<Height, Error> {
        match text.to_lowercase().as_str() {
            "sheer"  => Ok(Height::Sheer),
            "wale"   => Ok(Height::Wale),
            "rabbet" => Ok(Height::Rabbet),
            text     => {
                let feet = Feet::parse(text)
                    .context("Was unable to read height.")?;
                Ok(Height::Height(feet.into()))
            }
        }
    }
}

impl FromStr for Breadth {
    type Err = Error;
    fn from_str(text: &str) -> Result<Breadth, Error> {
        match text.to_lowercase().as_str() {
            "sheer" => Ok(Breadth::Sheer),
            text    => {
                let feet = Feet::parse(text)
                    .context("Was unable to read breadth.")?;
                Ok(Breadth::Breadth(feet.into()))
            }
        }
    }
}

impl FromStr for Diagonal {
    type Err = Error;
    fn from_str(text: &str) -> Result<Diagonal, Error> {
        match text.to_lowercase().as_str() {
            "a" => Ok(Diagonal::A),
            "b" => Ok(Diagonal::B),
            _ => bail!(concat!(
                "Could not read diagonal {}. Expected A or B."), text)
        }
    }
}
