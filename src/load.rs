//! Read in ship data from csv files.

use std::path::Path;
use std::str::FromStr;
use std::iter;
use std::io;
use std::fs;

use csv;
use failure::{Error, ResultExt};

use unit::*;
use spec::*;

/// Read the data file, which contains reference points along cross
/// sections of the hull.
pub fn read_data(path: &Path) -> Result<Data, Error> {
    let reader = open_csv_file(path)?;
    Ok(read_data_from_csv(reader)
        .context(format!("Could not parse file {:?}.", path))?)
}

/// Read the plank file, which says where planks should be placed on the hull.
pub fn read_planks(path: &Path) -> Result<Planks, Error> {
    let reader = open_csv_file(path)?;
    Ok(read_planks_from_csv(reader)
        .context(format!("Could not parse file {:?}.", path))?)
}

/// Read the configuration file.
pub fn read_config(path: &Path) -> Result<Config, Error> {
    let reader = open_csv_file(path)?;
    Ok(read_config_from_csv(reader)
        .context(format!("Could not parse file {:?}.", path))?)
}

fn open_csv_file(path: &Path) -> Result<csv::Reader<fs::File>, Error> {
    Ok(csv::Reader::from_path(path)
        .context(format!("Could not read file {:?}.", path))?)
}

fn read_planks_from_csv<T>(mut csv: csv::Reader<T>) -> Result<Planks, Error>
where
    T: io::Read,
{
    println!("Parsing planks file.");
    let mut stations = vec![];
    {
        let headers = csv.headers();
        let headers = headers.expect("Could not read stations.");
        let headers = headers.iter().skip(1);
        for header in headers {
            stations.push(read_plank_station(header));
        }
    }

    // Read plank curve fractions
    let mut planks = vec![];
    for row in csv.records() {
        let row = row?;
        let mut plank = vec![];
        for cell in row.iter().skip(1) {
            plank.push(read_plank_curve_fraction(cell)?);
        }
        planks.push(plank);
    }
    Ok(Planks {
        stations: stations,
        planks_locations: planks,
    })
}

fn read_config_from_csv<T>(mut csv: csv::Reader<T>) -> Result<Config, Error>
where
    T: io::Read,
{
    println!("Parsing config file.");
    match csv.deserialize().next() {
        None => bail!("Found no rows in config file."),
        Some(row) => Ok(row?),
    }
}

fn read_data_from_csv<T>(mut csv: csv::Reader<T>) -> Result<Data, Error>
where
    T: io::Read,
{
    // Read stations
    println!("Parsing stations.");
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
    let mut diagonals = vec![];
    loop {
        match read_section_name(&mut recs)? {
            None => break,
            Some(section) => {
                println!("Parsing section {:?}.", section);
                match section {
                    Section::Positions => {
                        read_section(&mut recs, &mut positions)
                    }
                    Section::Heights => read_section(&mut recs, &mut heights),
                    Section::Breadths => read_section(&mut recs, &mut breadths),
                    Section::Diagonals => {
                        read_section(&mut recs, &mut diagonals)
                    }
                }
            }.context(format!(
                "Could not parse section {:?}.",
                section
            ))?,
        };
    }

    Ok(Data {
        stations: stations,
        positions: positions,
        heights: heights,
        breadths: breadths,
        diagonals: diagonals,
    })
}

fn is_data_row<CSV>(csv: &mut iter::Peekable<CSV>) -> bool
where
    CSV: Iterator<Item = csv::Result<csv::StringRecord>>,
{
    match csv.peek() {
        None => false,
        Some(&Err(_)) => false,
        Some(&Ok(ref row)) => row.len() >= 2 && row.iter().nth(2) != Some(""),
    }
}

fn read_section<CSV, T>(
    csv: &mut iter::Peekable<CSV>,
    table: &mut Vec<DataRow<T>>,
) -> Result<(), Error>
where
    CSV: Iterator<Item = csv::Result<csv::StringRecord>>,
    T: FromStr<Err = Error>,
{
    loop {
        if !is_data_row(csv) {
            break;
        }
        let csv_row = csv.next();
        let csv_row = csv_row.expect("Could not parse row.")?;
        let mut csv_row = csv_row.iter();
        let head = csv_row
            .next()
            .expect("Could not parse first column of row.");

        let mut row = vec![];
        for csv_cell in csv_row {
            let cell = Feet::parse_opt(csv_cell)?;
            row.push(cell);
        }
        table.push((T::from_str(head)?, row));
    }
    Ok(())
}

fn read_section_name<CSV>(csv: &mut CSV) -> Result<Option<Section>, Error>
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
                    "diagonal" => Ok(Some(Section::Diagonals)),
                    _ => bail!(
                        concat!(
                            "Did not recognize the name {}. ",
                            "Expected one of these section names: ",
                            "Height, Breadth, Diagonal, Fore-Aft Position."
                        ),
                        name
                    ),
                },
                None => bail!("Expected section name, found blank line."),
            }
        }
        None => Ok(None),
    }
}

fn read_plank_curve_fraction(text: &str) -> Result<Option<f32>, Error> {
    if text == "x" {
        Ok(None)
    } else {
        let frac = f32::from_str(text)?;
        if frac >= 0.0 && frac <= 1.0 {
            Ok(Some(frac))
        } else {
            bail!(
                concat!(
                    "Plank location fractions must be between 0 ",
                    "and 1. Read fraction {}."
                ),
                frac
            )
        }
    }
}

fn read_plank_station(text: &str) -> PlankStation {
    match Feet::parse(text) {
        Ok(feet) => PlankStation::Position(feet),
        Err(_) => PlankStation::Station(text.to_string()),
    }
}

#[derive(Debug)]
enum Section {
    Positions,
    Heights,
    Breadths,
    Diagonals,
}
