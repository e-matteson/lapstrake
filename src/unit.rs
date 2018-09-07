//! Units. Right now only feet are supported.

use std::fmt;
use std::str::FromStr;

use error::{LapstrakeError, ResultExt};

/// Feet, inches, and eighths of an inch.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Feet {
    pub feet: u32,
    pub inches: u32,
    pub eighths: u32,
}

impl Feet {
    pub fn zero() -> Feet {
        Feet {
            feet: 0,
            inches: 0,
            eighths: 0,
        }
    }

    /// Parse Feet from a string, using the format 2-3-4.
    pub fn parse(text: &str) -> Result<Feet, LapstrakeError> {
        match Feet::parse_opt(text)? {
            None => Err(LapstrakeError::Load
                .context("this required measurement was omitted")),
            Some(feet) => Ok(feet),
        }
    }

    /// Parse Option<Feet> from a string, using the format 2-3-4, or
    /// "x" for None.
    pub fn parse_opt(text: &str) -> Result<Option<Feet>, LapstrakeError> {
        if text == "x" {
            return Ok(None);
        }

        let feet = Self::from_text(text).with_context(|| {
            format!(
                concat!(
                    "Was not able to read measurement '{}'. ",
                    "Expected formatting like 3-4-5 for 3' 4 5/8\". ",
                    "All parts of the measurement must be included, ",
                    "even if they are zero."
                ),
                text
            )
        })?;

        Ok(Some(feet))
    }

    fn from_text(text: &str) -> Result<Self, LapstrakeError> {
        let parts: Vec<&str> = text.split('-').collect();

        match parts.as_slice() {
            &[feet_str, inches_str, eighths_str] => Ok(Feet {
                feet: Self::parse_usize(feet_str).context("Invalid feet.")?,
                inches: Self::parse_usize(inches_str)
                    .context("Invalid inches.")?,
                eighths: Self::parse_usize(eighths_str)
                    .context("Invalid eighths.")?,
            }),
            _ => Err(LapstrakeError::Load
                .context("Didn't find exactly 3 parts in the measurement.")),
        }
    }

    fn parse_usize(text: &str) -> Result<u32, LapstrakeError> {
        u32::from_str(text).map_err(|_| {
            LapstrakeError::Load.with_context(|| {
                format!("Unable to read number in measurement: '{}'.", text)
            })
        })
    }
}

impl Into<f32> for Feet {
    fn into(self) -> f32 {
        (self.feet as f32)
            + (self.inches as f32 / 12.)
            + (self.eighths as f32 / 12. / 8.)
    }
}

impl fmt::Debug for Feet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{}-{}", self.feet, self.inches, self.eighths)
    }
}

impl fmt::Display for Feet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match (self.feet, self.inches, self.eighths) {
            (0, 0, 0) => write!(f, "0'"),
            (0, inches, 0) => write!(f, "{}\"", inches),
            (0, inches, eighths) => write!(f, "{} {}/8\"", inches, eighths),
            (feet, inches, 0) => write!(f, "{}' {}\"", feet, inches),
            (feet, inches, eighths) => {
                write!(f, "{}' {} {}/8\"", feet, inches, eighths)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_units() {
        let x = Feet {
            feet: 2,
            inches: 3,
            eighths: 4,
        };

        // Parsing
        assert_eq!(Feet::parse("2-3-4").unwrap(), x);

        // Debug printing
        assert_eq!(&format!("{:?}", x), "2-3-4");

        // Fancy printing
        assert_eq!(
            &format!(
                "{}",
                Feet {
                    feet: 0,
                    inches: 0,
                    eighths: 0,
                }
            ),
            "0'"
        );
        assert_eq!(
            &format!(
                "{}",
                Feet {
                    feet: 0,
                    inches: 30,
                    eighths: 0,
                }
            ),
            "30\""
        );
        assert_eq!(
            &format!(
                "{}",
                Feet {
                    feet: 0,
                    inches: 3,
                    eighths: 4,
                }
            ),
            "3 4/8\""
        );
        assert_eq!(
            &format!(
                "{}",
                Feet {
                    feet: 2,
                    inches: 0,
                    eighths: 5,
                }
            ),
            "2' 0 5/8\""
        );
        assert_eq!(
            &format!(
                "{}",
                Feet {
                    feet: 0,
                    inches: 0,
                    eighths: 6,
                }
            ),
            "0 6/8\""
        );
    }
}
