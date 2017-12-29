use unit::*;


pub struct Spec {
    pub config: Config,
    pub heights:   Vec<(Height,   Vec<usize>)>,
    pub breadths:  Vec<(Breadth,  Vec<usize>)>,
    pub diagonals: Vec<(Diagonal, Vec<usize>)>,
}

pub struct Config {
    pub sheer_thickness: usize,
    pub wale_thickness: usize
}

pub enum Height {
    Sheer,
    Wale,
    Rabbet,
    Height(usize)
}

pub enum Breadth {
    Sheer,
    Breadth(usize)
}

pub enum Diagonal {
    A,
    B
}

impl Height {
    pub fn parse(text: &str) -> Height {
        match text.to_lowercase().as_str() {
            "sheer"  => Height::Sheer,
            "wale"   => Height::Wale,
            "rabbet" => Height::Rabbet,
            text     => Height::Height(Feet::parse(text).into())
        }
    }
}

impl Breadth {
    pub fn parse(text: &str) -> Breadth {
        match text.to_lowercase().as_str() {
            "sheer" => Breadth::Sheer,
            text    => Breadth::Breadth(Feet::parse(text).into())
        }
    }
}

impl Diagonal {
    pub fn parse(text: &str) -> Diagonal {
        match text.to_lowercase().as_str() {
            "a" => Diagonal::A,
            "b" => Diagonal::B,
            _ => panic!(concat!(
                "Could not read diagonal {}. Expected A or B."), text)
        }
    }
}
