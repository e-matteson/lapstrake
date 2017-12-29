use std::fmt;
use std::str::FromStr;
use failure::{Error, ResultExt};


#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Feet {
    pub feet: u32,
    pub inches: u32,
    pub eighths: u32
}

impl Feet {
    pub fn parse(text: &str) -> Result<Feet, Error> {
        match Feet::parse_opt(text)? {
            None => bail!(concat!(
                "Was unable to read measurement '{}'. ",
                "(This measurement cannot be ommited.)"),
                          text),
            Some(feet) => Ok(feet)
        }
    }

    pub fn parse_opt(text: &str) -> Result<Option<Feet>, Error> {

        fn parse_usize(text: &str) -> Result<u32, Error> {
            match u32::from_str(text) {
                Ok(n) => Ok(n),
                Err(_) => bail!(
                    "Was not able to read number in measurement '{}'.",
                    text)
            }
        }

        if text == "x" {
            return Ok(None);
        }

        let parts: Vec<&str> = text.split('-').collect();
        let ctx = concat!(
            "Was not able to read measurement. ",
            "Expected formatting like 3-4-5 for 3' 4 5/8\".");
            
        match parts.as_slice() {
            &[feet, inches, eighths] =>
                Ok(Some(Feet{
                    feet:    parse_usize(&feet).context(ctx)?,
                    inches:  parse_usize(&inches).context(ctx)?,
                    eighths: parse_usize(&eighths).context(ctx)?
                })),
            _ => bail!(concat!(
                "Was not able to read measurement '{}'. ",
                "Expected formatting like 3-4-5 for 3' 4 5/8\". ",
                "All parts of the measurement must be included, ",
                "even if they are zero."),
                        text)
        }
    }
}


impl Into<usize> for Feet {
    fn into(self) -> usize {
        self.feet as usize * 12 * 8
            + self.inches as usize * 8
            + self.eighths as usize
    }
}

impl Into<Feet> for usize {
    fn into(self) -> Feet {
        Feet{
            feet:    ((self / 8) / 12) as u32,
            inches:  ((self / 8) % 12) as u32,
            eighths: (self % 8) as u32
        }
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
            (0, 0, 0) =>
                write!(f, "0'"),
            (0, inches, 0) =>
                write!(f, "{}\"", inches),
            (0, inches, eighths) =>
                write!(f, "{} {}/8\"", inches, eighths),
            (feet, inches, 0) =>
                write!(f, "{}' {}\"", feet, inches),
            (feet, inches, eighths) =>
                write!(f, "{}' {} {}/8\"", feet, inches, eighths)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_units() {
        let x = Feet{
            feet: 2,
            inches: 3,
            eighths: 4
        };

        // Conversion to/from usize
        let x_usize: usize = x.into();
        assert_eq!(x_usize, 220);
        let x_feet: Feet = x_usize.into();
        assert_eq!(x_feet, x);

        // Parsing
        assert_eq!(Feet::parse("2-3-4"), x);

        // Debug printing
        assert_eq!(&format!("{:?}", x), "2-3-4");

        // Fancy printing
        assert_eq!(&format!("{}", Feet{ feet: 0, inches: 0, eighths: 0 }),
                   "0'");
        assert_eq!(&format!("{}", Feet{ feet: 0, inches: 30, eighths: 0 }),
                   "30\"");
        assert_eq!(&format!("{}", Feet{ feet: 0, inches: 3, eighths: 4 }),
                   "3 1/2\"");
        assert_eq!(&format!("{}", Feet{ feet: 2, inches: 0, eighths: 5 }),
                   "2' 0 5/8\"");
        assert_eq!(&format!("{}", Feet{ feet: 0, inches: 0, eighths: 6 }),
                   "0 3/4\"");
    }
}
