/// Example:
///
/// ```
/// let path = SvgPath::new(vec![P2::new(0., 0.), P2::new(50., 50.), P2::new(100., 20.)])
///     .style(PathStyle2::LineWithDots)
///     .stroke(SvgColor::Blue, 2.);
/// path.save("tests/tmp/bluepath.svg").unwrap();
/// ```
///

use scad_dots::utils::{Axis, P2, V2};
use scad_dots::core::MinMaxCoord;

use failure::Error;

use svg::{self, Document, Node};
use svg::node::Value;
use svg::node::element::{Circle, Group, Path, Rectangle};
use svg::node::element::path::Data;

const SCALE: f32 = 96.; // pixels per inch

#[derive(Clone, Copy, Debug)]
pub enum PathStyle2 {
    Dots,
    Line,
    LineWithDots,
}

pub struct SvgDoc {
    contents: Group,
    bound: Bound,
}

pub struct SvgPath {
    points: Vec<P2>,
    stroke: Stroke,
    style: PathStyle2,
}

#[derive(Clone, Copy, Debug)]
pub struct SvgCircle {
    pos: P2,
    radius: f32,
    fill: SvgColor,
}

#[derive(Clone, Copy, Debug)]
pub struct SvgRect {
    pos: P2,
    size: V2,
    stroke: Option<Stroke>,
    fill: Option<SvgColor>,
    fillet: Option<V2>,
}

#[derive(Clone, Copy, Debug)]
pub struct Bound {
    low: P2,
    high: P2,
}

#[derive(Clone, Copy, Debug)]
struct Stroke {
    color: SvgColor,
    width: f32,
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
    pub fn new() -> SvgDoc {
        SvgDoc {
            contents: Group::new(),
            bound: Bound::new(),
        }
    }

    // pub fn append_node<T>(&mut self, node: T)
    // where
    //     T: Node,
    // {
    //     self.contents.append(node)
    // }

    pub fn append_path(&mut self, path: SvgPath) {
        self.bound = self.bound.combine(path.bound());
        self.contents.append(path.finalize())
    }

    pub fn append_paths(&mut self, paths: Vec<SvgPath>) {
        for path in paths {
            self.append_path(path);
        }
    }

    // pub fn set_bound(&mut self, bound: Bound) {
    //     // If you've only used append_path, you won't need to set the bound explicitly
    //     self.bound = bound;
    // }

    pub fn finalize(self) -> Document {
        let background = SvgRect::new(self.bound.low, self.bound.size())
            .fill(SvgColor::White);
        let doc = Document::new()
            .set("viewBox", self.bound.view_box())
            .add(background.finalize())
            .add(self.contents);
        doc
    }

    pub fn save(self, filename: &str) -> Result<(), Error> {
        println!("Saving svg file {}.", filename);
        Ok(svg::save(filename, &self.finalize())?)
    }
}

impl SvgPath {
    pub fn new(points: Vec<P2>) -> SvgPath {
        SvgPath {
            points: points.into_iter().map(|p| SCALE * p).collect(),
            stroke: Stroke {
                color: SvgColor::Black,
                width: 1.,
            },
            style: PathStyle2::Line,
        }
    }

    pub fn new_segment(start: P2, end: P2) -> SvgPath {
        SvgPath::new(vec![start, end])
    }

    pub fn stroke(mut self, color: SvgColor, width: f32) -> Self {
        self.stroke = Stroke {
            color: color,
            width: width,
        };
        self
    }

    pub fn style(mut self, style: PathStyle2) -> Self {
        self.style = style;
        self
    }

    pub fn finalize(self) -> Group {
        let mut group = Group::new();
        if self.style.has_line() {
            let mut path = Path::new();
            path.assign("d", self.path_data());
            path.assign("stroke", self.stroke.color);
            path.assign("stroke-width", self.stroke.width);
            path.assign("fill", "none");
            group.append(path);
        }

        if self.style.has_dots() {
            group.append(self.dots())
        }
        group
    }

    pub fn save(self, filename: &str) -> Result<(), Error> {
        let mut doc = SvgDoc::new();
        doc.append_path(self);
        doc.save(filename)?;
        Ok(())
    }

    fn dots(&self) -> Group {
        let radius = self.stroke.width;
        let color = self.stroke.color;

        let mut group = Group::new();
        for p in &self.points {
            group
                .append(SvgCircle::new(p.to_owned(), radius, color).finalize());
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

impl PathStyle2 {
    fn has_line(&self) -> bool {
        match *self {
            PathStyle2::Dots => false,
            PathStyle2::Line => true,
            PathStyle2::LineWithDots => true,
        }
    }
    fn has_dots(&self) -> bool {
        match *self {
            PathStyle2::Dots => true,
            PathStyle2::Line => false,
            PathStyle2::LineWithDots => true,
        }
    }
}

impl SvgCircle {
    pub fn new(pos: P2, radius: f32, color: SvgColor) -> Self {
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

impl SvgRect {
    pub fn new(pos: P2, size: V2) -> Self {
        Self {
            pos: pos,
            size: size,
            stroke: None,
            fill: None,
            fillet: None,
        }
    }

    // pub fn stroke(mut self, color: SvgColor, width: f32) -> Self {
    //     self.stroke = Some(Stroke {
    //         color: color,
    //         width: width,
    //     });
    //     self
    // }

    pub fn fill(mut self, fill: SvgColor) -> Self {
        self.fill = Some(fill);
        self
    }

    // pub fn fillet(mut self, radius: f32) -> Self {
    //     assert!(radius >= 0.);
    //     self.fillet = Some(V2::new(radius, radius));
    //     self
    // }

    // pub fn center(&self) -> P2 {
    //     self.pos + self.size / 2.
    // }

    // pub fn scale(mut self, factor: f32) -> Self {
    //     // Scale the switch uniformly around its center
    //     let center = self.center();
    //     self.size *= factor;
    //     self.pos = center - self.size / 2.;
    //     self
    // }

    pub fn finalize(self) -> Rectangle {
        let mut element = Rectangle::new()
            .set("x", self.pos.x)
            .set("y", self.pos.y)
            .set("width", self.size.x)
            .set("height", self.size.y);

        if let Some(stroke) = self.stroke {
            element.assign("stroke", stroke.color);
            element.assign("stroke-width", stroke.width);
        }

        if let Some(color) = self.fill {
            element.assign("fill", color);
        } else {
            element.assign("fill", "none");
        }

        if let Some(fillet) = self.fillet {
            element.assign("rx", fillet.x);
            element.assign("ry", fillet.y);
        }
        element
    }
}

impl Bound {
    fn new() -> Bound {
        Bound::from_origin(0., 0.)
    }

    fn from_origin(width: f32, height: f32) -> Bound {
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
    fn size(&self) -> V2 {
        V2::new(self.width(), self.height())
    }

    fn combine(&self, other: Bound) -> Bound {
        Bound {
            low: P2::new(
                self.low.x.min(other.low.x),
                self.low.y.min(other.low.y),
            ),
            high: P2::new(
                self.high.x.max(other.high.x),
                self.high.y.max(other.high.y),
            ),
        }
    }
}

// #[test]
// fn test_svg() {
//     let data = Data::new()
//         .move_to((10, 10))
//         .line_by((0, 50))
//         .line_by((50, 0))
//         .line_by((0, -50))
//         .close();

//     let path = Path::new()
//         .set("fill", "none")
//         .set("stroke", "black")
//         .set("stroke-width", 3)
//         .set("d", data);

//     let document = Document::new().set("viewBox", (0, 0, 70, 70)).add(path);
//     assert_eq!(
//         "<svg viewBox=\"0 0 70 70\" xmlns=\"http://www.w3.org/2000/svg\">
// <path d=\"M10,10 l0,50 l50,0 l0,-50 z\" fill=\"none\" stroke=\"black\" stroke-width=\"3\"/>
// </svg>",
//         document.to_string()
//     );
// }

impl Into<Value> for SvgColor {
    fn into(self) -> Value {
        match self {
            // SvgColor::Red => "#fa99b7",
            SvgColor::Red => "red",
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
