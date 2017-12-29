use std::fmt;
use std::str::FromStr;


#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Feet {
    pub feet: u32,
    pub inches: u32,
    pub eighths: u32
}

impl Feet {
    fn parse(text: &str) -> Feet {

        fn parse_usize(text: &str) -> u32 {
            match u32::from_str(text) {
                Ok(n) => n,
                Err(_) => panic!(concat!(
                    "Was not able to read number in measurement: {}. ",
                    "Expected formatting like 3-4-5 for 3' 4 5/8\"."),
                                 text)
            }
        }

        let parts: Vec<&str> = text.split('-').collect();
        match parts.as_slice() {
            &[feet, inches, eighths] =>
                Feet{
                    feet:    parse_usize(&feet),
                    inches:  parse_usize(&inches),
                    eighths: parse_usize(&eighths)
                },
            _ => panic!(concat!(
                "Was not able to read measurement: {}. ",
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
                write!(f, "{} {}\"", inches, Eighths(eighths)),
            (feet, inches, 0) =>
                write!(f, "{}' {}\"", feet, inches),
            (feet, inches, eighths) => {
                write!(f, "{}' {} {}\"", feet, inches, Eighths(eighths))
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Eighths(u32);

impl fmt::Display for Eighths {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Eighths(eighths) = *self;
        match eighths {
            1 | 3 | 5 | 7 => write!(f, "{}/8", eighths),
            2 | 6         => write!(f, "{}/4", eighths / 2),
            4             => write!(f, "1/2"),
            0             => write!(f, "0/8"), // shouldn't be called
            _ => panic!("Bad number of eighths! {}/8", eighths)
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
