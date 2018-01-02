//! Units. Right now only feet are supported.

use std::fmt;
use std::str::FromStr;
use failure::{Error, ResultExt};

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
    pub fn parse(text: &str) -> Result<Feet, Error> {
        match Feet::parse_opt(text)? {
            None => bail!(
                concat!(
                    "Was unable to read measurement '{}'. ",
                    "(This measurement cannot be ommited.)"
                ),
                text
            ),
            Some(feet) => Ok(feet),
        }
    }

    /// Parse Option<Feet> from a string, using the format 2-3-4, or
    /// "x" for None.
    pub fn parse_opt(text: &str) -> Result<Option<Feet>, Error> {
        fn parse_usize(text: &str) -> Result<u32, Error> {
            match u32::from_str(text) {
                Ok(n) => Ok(n),
                Err(_) => bail!(
                    "Was not able to read number in measurement '{}'.",
                    text
                ),
            }
        }

        if text == "x" {
            return Ok(None);
        }

        let parts: Vec<&str> = text.split('-').collect();
        let message = concat!(
            "Was not able to read measurement. ",
            "Expected formatting like 3-4-5 for 3' 4 5/8\"."
        );

        match parts.as_slice() {
            &[feet, inches, eighths] => Ok(Some(Feet {
                feet: parse_usize(&feet).context(message)?,
                inches: parse_usize(&inches).context(message)?,
                eighths: parse_usize(&eighths).context(message)?,
            })),
            _ => bail!(
                concat!(
                    "Was not able to read measurement '{}'. ",
                    "Expected formatting like 3-4-5 for 3' 4 5/8\". ",
                    "All parts of the measurement must be included, ",
                    "even if they are zero."
                ),
                text
            ),
        }
    }
}

impl Into<f32> for Feet {
    fn into(self) -> f32 {
        (self.feet as f32) + (self.inches as f32 / 12.)
            + (self.eighths as f32 / 12. / 8.)
    }
}

// impl Into<Feet> for usize {
//     fn into(self) -> Feet {
//         Feet {
//             feet: ((self / 8) / 12) as u32,
//             inches: ((self / 8) % 12) as u32,
//             eighths: (self % 8) as u32,
//         }
//     }
// }

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
