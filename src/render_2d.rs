use scad_dots::utils::{Axis, P2, V2};
use scad_dots::core::MinMaxCoord;

use failure::Error;

use svg::{self, Document, Node};
use svg::node::Value;
use svg::node::element::{Circle, Group, Path};
use svg::node::element::path::Data;


/// Example:
///
/// ```
/// let path = SvgPath::new(vec![P2::new(0., 0.), P2::new(50., 50.), P2::new(100., 20.)])
///     .show_points()
///     .stroke(SvgColor::Blue, 2.);
/// path.save("tests/tmp/bluepath.svg").unwrap();
/// ```
///

pub struct SvgDoc {
    doc: Document,
}

pub struct SvgPath {
    points: Vec<P2>,
    show_points: bool,
    stroke: Stroke,
}

#[derive(Clone, Copy, Debug)]
pub struct SvgCircle {
    pos: P2,
    radius: f64,
    fill: SvgColor,
}

#[derive(Clone, Copy, Debug)]
pub struct Bound {
    low: P2,
    high: P2,
}


#[derive(Clone, Copy, Debug)]
struct Stroke {
    color: SvgColor,
    width: f64,
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum SvgColor {
    Red,
    Yellow,
    Green,
    Cyan,
    Blue,
    Magenta,
    Black,
    White,
    LightGrey,
    DarkGrey,
}

impl SvgDoc {
    pub fn new(view: Bound) -> SvgDoc {
        let mut doc = Document::new().set("viewBox", view.view_box());
        SvgDoc { doc: doc }
    }

    pub fn append<T>(&mut self, node: T)
    where
        T: Node,
    {
        self.doc.append(node)
    }

    pub fn save(&self, path: &str) -> Result<(), Error> {
        Ok(svg::save(path, &self.doc)?)
    }
}

impl SvgPath {
    pub fn new(points: Vec<P2>) -> SvgPath {
        SvgPath {
            points: points,
            show_points: false,
            stroke: Stroke {
                color: SvgColor::Black,
                width: 1.,
            },
        }
    }

    pub fn show_points(mut self) -> SvgPath {
        self.show_points = true;
        self
    }

    pub fn stroke(mut self, color: SvgColor, width: f64) -> Self {
        self.stroke = Stroke {
            color: color,
            width: width,
        };
        self
    }

    pub fn finalize(self) -> Group {
        let mut path = Path::new();
        path.assign("d", self.path_data());
        path.assign("stroke", self.stroke.color);
        path.assign("stroke-width", self.stroke.width);
        path.assign("fill", "none");

        let mut group = Group::new().add(path);
        if self.show_points {
            group.append(self.dots())
        }
        group
    }

    pub fn save(self, path: &str) -> Result<(), Error> {
        let mut doc = SvgDoc::new(self.bound());
        doc.append(self.finalize());
        doc.save(path)?;
        Ok(())
    }

    fn dots(&self) -> Group {
        let radius = self.stroke.width;
        let color = self.stroke.color;

        let mut group = Group::new();
        for p in &self.points {
            group.append(SvgCircle::new(p.to_owned(), radius, color).finalize());
        }
        group
    }

    fn path_data(&self) -> Data {
        let mut data = Data::new();
        let mut points = self.points.iter();
        let first = points.next().expect("path is empty");
        data = data.move_to(to_tuple(first));
        for p in points {
            data = data.line_to(to_tuple(p));
        }
        data
    }

    fn bound(&self) -> Bound {
        let view_scale = 1.2;
        let center = self.points.midpoint2();
        let size = view_scale
            * V2::new(
                self.points.bound_length(Axis::X),
                self.points.bound_length(Axis::Y),
            );
        Bound {
            low: center - size / 2.,
            high: center + size / 2.,
        }
    }
}

impl SvgCircle {
    pub fn new(pos: P2, radius: f64, color: SvgColor) -> Self {
        Self {
            pos: pos,
            radius: radius,
            fill: color,
        }
    }

    pub fn finalize(self) -> Circle {
        let mut element = Circle::new()
            .set("cx", self.pos.x)
            .set("cy", self.pos.y)
            .set("r", self.radius);

        element.assign("fill", self.fill);
        element
    }
}

impl Bound {
    fn from_origin(&self, width: f32, height: f32) -> Bound {
        Bound {
            low: P2::origin(),
            high: P2::new(width, height),
        }
    }
    fn view_box(&self) -> (f32, f32, f32, f32) {
        (self.low.x, self.low.y, self.width(), self.height())
    }
    fn width(&self) -> f32 {
        self.high.x - self.low.x
    }
    fn height(&self) -> f32 {
        self.high.y - self.low.y
    }
    fn combine(&self, other: Bound) -> Bound {
        Bound {
            low: P2::new(self.low.x.min(other.low.x), self.low.y.min(other.low.y)),
            high: P2::new(self.high.x.max(other.high.x), self.high.y.max(other.high.y)),
        }
    }
}


#[test]
fn test_svg() {
    let data = Data::new()
        .move_to((10, 10))
        .line_by((0, 50))
        .line_by((50, 0))
        .line_by((0, -50))
        .close();

    let path = Path::new()
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 3)
        .set("d", data);

    let document = Document::new().set("viewBox", (0, 0, 70, 70)).add(path);
    assert_eq!(
        "<svg viewBox=\"0 0 70 70\" xmlns=\"http://www.w3.org/2000/svg\">
<path d=\"M10,10 l0,50 l50,0 l0,-50 z\" fill=\"none\" stroke=\"black\" stroke-width=\"3\"/>
</svg>",
        document.to_string()
    );
}


impl Into<Value> for SvgColor {
    fn into(self) -> Value {
        match self {
            SvgColor::Red => "#fa99b7",
            SvgColor::Yellow => "#eba676",
            SvgColor::Green => "#a7be74",
            SvgColor::Cyan => "#48c9b4",
            SvgColor::Blue => "#3ac3f5",
            SvgColor::Magenta => "#b9acf6",
            SvgColor::Black => "#000000",
            SvgColor::White => "#ffffff",
            SvgColor::LightGrey => "#eeeeee",
            SvgColor::DarkGrey => "#b6b6b6",
        }.into()
    }
}

fn to_tuple(pos: &P2) -> (f32, f32) {
    (pos.x, pos.y)
}
