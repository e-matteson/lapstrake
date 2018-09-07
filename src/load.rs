//! Read in ship data from csv files.

use std::fs;
use std::io;
use std::iter;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use csv;

use error::{LapstrakeError, ResultExt};
use spec::*;
use unit::*;

#[derive(Debug)]
enum Section {
    Positions,
    Heights,
    Breadths,
}

impl Spec {
    /// Read the three spec files from a given directory.
    /// data.csv contains reference points along cross sections of the hull.
    /// planks.csv says where planks should be placed on the hull.
    /// config.csv has various configuration options.
    pub fn load_from(directory: &Path) -> Result<Spec, LapstrakeError> {
        let path_to = |filename: &str| {
            let mut path = directory.to_path_buf();
            path.push(filename);
            path
        };

        let data = Data::load_from(&path_to("data.csv"))
            .context("failed to load data sheet")?;

        let planks = Planks::load_from(&path_to("planks.csv"))
            .context("failed to load planks sheet")?;

        let config = Config::load_from(&path_to("config.csv"))
            .context("failed to load config sheet")?;

        Ok(Spec {
            data,
            planks,
            config,
        })
    }
}

impl Data {
    fn load_from(file: &Path) -> Result<Data, LapstrakeError> {
        let mut csv = open_csv_file(file)?;
        // Read stations
        let mut stations = vec![];
        {
            let headers = csv.headers();
            let headers = headers.expect("Could not read stations.");
            let headers = headers.iter().skip(1);
            for header in headers {
                stations.push(header.to_string());
            }
        }

        let mut recs = csv.records().peekable();

        // Read Sections
        let mut positions = vec![];
        let mut heights = vec![];
        let mut breadths = vec![];
        loop {
            match Self::read_section_name(&mut recs)? {
                None => break,
                Some(section) => {
                    match section {
                        Section::Positions => {
                            Self::load_section(&mut recs, &mut positions)
                        }
                        Section::Heights => {
                            Self::load_section(&mut recs, &mut heights)
                        }
                        Section::Breadths => {
                            Self::load_section(&mut recs, &mut breadths)
                        }
                    }
                }.with_context(|| {
                    format!("Could not parse section {:?}.", section)
                })?,
            }
        }

        Ok(Data {
            stations,
            positions,
            heights,
            breadths,
        })
    }

    fn load_section<CSV, T>(
        csv: &mut iter::Peekable<CSV>,
        table: &mut Vec<DataRow<T>>,
    ) -> Result<(), LapstrakeError>
    where
        CSV: Iterator<Item = csv::Result<csv::StringRecord>>,
        T: FromStr<Err = LapstrakeError>,
    {
        loop {
            if !Self::is_data_row(csv) {
                break;
            }
            let csv_row = csv.next().ok_or_else(|| {
                LapstrakeError::load("Failed to get next row.")
            })??;
            let mut csv_row = csv_row.iter();

            let head = csv_row.next().ok_or_else(|| {
                LapstrakeError::load("Failed to get first column of row.")
            })?;

            let mut row = vec![];
            for csv_cell in csv_row {
                let cell = Feet::parse_opt(csv_cell)?;
                row.push(cell);
            }
            table.push((T::from_str(head)?, row));
        }
        Ok(())
    }

    fn is_data_row<CSV>(csv: &mut iter::Peekable<CSV>) -> bool
    where
        CSV: Iterator<Item = csv::Result<csv::StringRecord>>,
    {
        match csv.peek() {
            None => false,
            Some(&Err(_)) => false,
            Some(&Ok(ref row)) => {
                row.len() >= 2 && row.iter().nth(2) != Some("")
            }
        }
    }

    fn read_section_name<CSV>(
        csv: &mut CSV,
    ) -> Result<Option<Section>, LapstrakeError>
    where
        CSV: Iterator<Item = csv::Result<csv::StringRecord>>,
    {
        match csv.next() {
            Some(row) => {
                let row = row?;
                let mut row = row.iter();
                match row.next() {
                    Some(name) => match name.to_lowercase().as_str() {
                        "fore-aft position" => Ok(Some(Section::Positions)),
                        "height" => Ok(Some(Section::Heights)),
                        "breadth" => Ok(Some(Section::Breadths)),
                        _ => Err(LapstrakeError::load(&format!(
                            concat!(
                                "Did not recognize the name {}. ",
                                "Expected one of these section names: ",
                                "Height, Breadth, Fore-Aft Position."
                            ),
                            name,
                        ))),
                    },
                    None => Err(LapstrakeError::load(
                        "Expected section name, found blank line.",
                    )),
                }
            }
            None => Ok(None),
        }
    }
}

impl Planks {
    fn load_from(file: &Path) -> Result<Planks, LapstrakeError> {
        let mut csv = open_csv_file(file)?;
        let mut stations = vec![];
        {
            let headers = csv.headers();
            let headers = headers.expect("Could not read stations.");
            let headers = headers.iter().skip(1);
            for header in headers {
                stations.push(Self::read_plank_station(header));
            }
        }

        // Read plank curve fractions
        let mut planks = vec![];
        for row in csv.records() {
            let row = row?;
            let mut plank = vec![];
            for cell in row.iter().skip(1) {
                plank.push(Self::read_plank_curve_fraction(cell)?);
            }
            planks.push(plank);
        }
        Ok(Planks {
            stations,
            plank_locations: planks,
        })
    }

    fn read_plank_curve_fraction(
        text: &str,
    ) -> Result<Option<f32>, LapstrakeError> {
        if text == "x" {
            Ok(None)
        } else {
            let frac = f32::from_str(text)
                .map_err(|e| LapstrakeError::Load(e.to_string()))?;

            if frac >= 0.0 && frac <= 1.0 {
                Ok(Some(frac))
            } else {
                Err(LapstrakeError::Load(format!(
                    concat!(
                        "Plank location fractions must be between 0 ",
                        "and 1. Read fraction {}."
                    ),
                    frac
                )))
            }
        }
    }

    fn read_plank_station(text: &str) -> PlankStation {
        match Feet::parse(text) {
            Ok(feet) => PlankStation::Position(feet),
            Err(_) => PlankStation::Station(text.to_string()),
        }
    }
}

impl Config {
    fn load_from(file: &Path) -> Result<Config, LapstrakeError> {
        let mut csv = open_csv_file(file)?;
        match csv.deserialize().next() {
            None => Err(LapstrakeError::load("Found no rows in config file.")),
            Some(row) => Ok(row?),
        }
    }
}

fn open_csv_file(path: &Path) -> Result<csv::Reader<fs::File>, LapstrakeError> {
    println!("Loading file: '{:?}'.", path);
    csv::Reader::from_path(&path)
        .map_err(|csv_err| LapstrakeError::from(csv_err))
        .with_context(|| format!("Could not read csv file: '{:?}'.", path))
}

impl FromStr for BreadthLine {
    type Err = LapstrakeError;
    fn from_str(text: &str) -> Result<BreadthLine, LapstrakeError> {
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
    type Err = LapstrakeError;
    fn from_str(text: &str) -> Result<HeightLine, LapstrakeError> {
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
