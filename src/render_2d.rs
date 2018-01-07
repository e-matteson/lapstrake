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

use svg::{self, node, Document, Node};
use svg::node::Value;
use svg::node::element::{Circle, Group, Path, Rectangle, Text};
use svg::node::element::path::Data;

// const SCALE: f32 = 96.; // pixels per inch
const SCALE: f32 = 1.; // pixels per inch

#[derive(Clone, Copy, Debug)]
pub enum PathStyle2 {
    Dots,
    Line,
    LineWithDots,
}

pub struct SvgDoc {
    contents: SvgGroup,
    // bound: Bound,
}

#[derive(Clone, Debug)]
pub struct SvgPath {
    points: Vec<P2>,
    stroke: Stroke,
    style: PathStyle2,
    is_closed: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct SvgCircle {
    pos: P2,
    radius: f32,
    stroke: Option<Stroke>,
    fill: Option<SvgColor>,
}

#[derive(Clone, Copy, Debug)]
pub struct SvgRect {
    pos: P2,
    size: V2,
    stroke: Option<Stroke>,
    fill: Option<SvgColor>,
    fillet: Option<V2>,
}

#[derive(Clone)]
pub struct SvgGroup {
    group: Group,
    bound: Option<Bound>,
    translation: Option<V2>,
}

#[derive(Clone, Copy, Debug)]
pub struct Bound {
    pub low: P2,
    pub high: P2,
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

#[derive(Clone, Debug)]
pub struct SvgText {
    pub lines: Vec<String>,
    pub pos: P2,
    pub color: SvgColor,
    pub size: f32,
}

pub trait Bounded {
    fn bound(&self) -> Option<Bound>;
}

pub trait ToSvg {
    type Output: Node;
    fn finalize(self) -> Self::Output;
}

impl SvgDoc {
    pub fn new() -> SvgDoc {
        SvgDoc {
            contents: SvgGroup::new(),
        }
    }

    pub fn append_node<T>(&mut self, node: T)
    where
        T: Node,
    {
        self.contents.append_node(node)
    }

    pub fn append<T>(&mut self, thing: T)
    where
        T: ToSvg + Bounded,
    {
        self.contents.append(thing)
    }

    pub fn append_vec<T>(&mut self, things: Vec<T>)
    where
        T: ToSvg + Bounded,
    {
        for thing in things {
            self.append(thing);
        }
    }

    pub fn save(self, filename: &str) -> Result<(), Error> {
        println!("Saving svg file {}.", filename);
        Ok(svg::save(filename, &self.finalize())?)
    }
}

impl ToSvg for SvgDoc {
    type Output = Document;

    fn finalize(self) -> Self::Output {
        let mut doc = Document::new();
        if let Some(bound) = self.bound() {
            let background =
                SvgRect::new(bound.low, bound.size()).fill(SvgColor::White);
            doc.append(background.finalize());
            doc.assign("viewBox", bound.view_box());
        }
        doc.append(self.contents.finalize());
        doc
    }
}

impl Bounded for SvgDoc {
    fn bound(&self) -> Option<Bound> {
        self.contents.bound
    }
}

impl SvgPath {
    pub fn new(points: Vec<P2>) -> SvgPath {
        SvgPath {
            points: points.into_iter().map(|p| SCALE * p).collect(),
            stroke: Stroke {
                color: SvgColor::Black,
                width: 1. * SCALE,
            },
            style: PathStyle2::Line,
            is_closed: false,
        }
    }

    pub fn new_segment(start: P2, end: P2) -> SvgPath {
        SvgPath::new(vec![start, end])
    }

    pub fn stroke(mut self, color: SvgColor, width: f32) -> Self {
        self.stroke = Stroke {
            color: color,
            width: width * SCALE,
        };
        self
    }

    pub fn style(mut self, style: PathStyle2) -> Self {
        self.style = style;
        self
    }

    pub fn close(mut self) -> Self {
        self.is_closed = true;
        self
    }

    pub fn append(&mut self, new_points: Vec<P2>) {
        self.points
            .extend(new_points.into_iter().map(|p| SCALE * p))
    }

    pub fn save(self, filename: &str) -> Result<(), Error> {
        let mut doc = SvgDoc::new();
        doc.append(self);
        doc.save(filename)?;
        Ok(())
    }

    fn dots(&self) -> Group {
        let radius = self.stroke.width;
        let color = self.stroke.color;

        let mut group = Group::new();
        for p in &self.points {
            group.append(
                SvgCircle::new(p.to_owned(), radius).fill(color).finalize(),
            );
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
        if self.is_closed {
            data = data.close();
        }
        data
    }
}

impl ToSvg for SvgPath {
    type Output = Group;

    fn finalize(self) -> Self::Output {
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
}

impl Bounded for SvgPath {
    fn bound(&self) -> Option<Bound> {
        let center = self.points.midpoint2();

        let size = V2::new(
            self.points.bound_length(Axis::X),
            self.points.bound_length(Axis::Y),
        );

        Some(Bound {
            low: center - size / 2.,
            high: center + size / 2.,
        })
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
    pub fn new(pos: P2, radius: f32) -> Self {
        Self {
            pos: pos * SCALE,
            radius: radius * SCALE,
            stroke: None,
            fill: None,
        }
    }

    pub fn stroke(mut self, color: SvgColor, width: f32) -> Self {
        self.stroke = Some(Stroke {
            color: color,
            width: width * SCALE,
        });
        self
    }

    pub fn fill(mut self, fill: SvgColor) -> Self {
        self.fill = Some(fill);
        self
    }
}

impl ToSvg for SvgCircle {
    type Output = Circle;

    fn finalize(self) -> Self::Output {
        let mut element = Circle::new()
            .set("cx", self.pos.x)
            .set("cy", self.pos.y)
            .set("r", self.radius);

        if let Some(stroke) = self.stroke {
            element.assign("stroke", stroke.color);
            element.assign("stroke-width", stroke.width);
        }

        if let Some(color) = self.fill {
            element.assign("fill", color);
        } else {
            element.assign("fill", "none");
        }

        element
    }
}

impl Bounded for SvgCircle {
    fn bound(&self) -> Option<Bound> {
        let offset = V2::new(self.radius, self.radius);
        Some(Bound {
            low: self.pos - offset,
            high: self.pos + offset,
        })
    }
}

impl SvgRect {
    pub fn new(pos: P2, size: V2) -> Self {
        Self {
            pos: pos * SCALE,
            size: size * SCALE,
            stroke: None,
            fill: None,
            fillet: None,
        }
    }

    pub fn stroke(mut self, color: SvgColor, width: f32) -> Self {
        self.stroke = Some(Stroke {
            color: color,
            width: width,
        });
        self
    }

    pub fn fill(mut self, fill: SvgColor) -> Self {
        self.fill = Some(fill);
        self
    }

    pub fn fillet(mut self, radius: f32) -> Self {
        let radius = radius * SCALE;
        assert!(radius >= 0.);
        self.fillet = Some(V2::new(radius, radius));
        self
    }

    pub fn center(&self) -> P2 {
        self.pos + self.size / 2.
    }

    pub fn scale(mut self, factor: f32) -> Self {
        // Scale the switch uniformly around its center
        let center = self.center();
        self.size *= factor;
        self.pos = center - self.size / 2.;
        self
    }
}

impl ToSvg for SvgRect {
    type Output = Rectangle;

    fn finalize(self) -> Self::Output {
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

impl Bounded for SvgRect {
    fn bound(&self) -> Option<Bound> {
        Some(Bound {
            low: self.pos,
            high: self.pos + self.size,
        })
    }
}

impl SvgGroup {
    pub fn new() -> SvgGroup {
        SvgGroup {
            group: Group::new(),
            bound: None,
            translation: None,
        }
    }

    pub fn new_grid(
        contents: Vec<SvgGroup>,
        spacing_factor: f32,
    ) -> Result<SvgGroup, Error> {
        let num_columns = (contents.len() as f32).sqrt() as usize;
        let mut group = SvgGroup::new();
        let mut column_bound = Bound::new();
        // for column in contents.chunks(num_columns) {
        for (i, mut sub_group) in contents.into_iter().enumerate() {
            sub_group
                .translate_to(column_bound.relative_pos(0., spacing_factor))?;
            column_bound = column_bound.union(sub_group.bound());
            group.append(sub_group);

            if i % num_columns == num_columns - 1 {
                column_bound = Bound::empty_at(
                    column_bound.relative_pos(spacing_factor, 0.),
                );
            }
        }
        Ok(group)
    }

    pub fn append<T>(&mut self, thing: T)
    where
        T: Bounded + ToSvg,
    {
        self.bound = if let Some(current_bound) = self.bound {
            Some(current_bound.union(thing.bound()))
        } else {
            thing.bound()
        };

        self.group.append(thing.finalize());
    }

    pub fn append_node<T>(&mut self, node: T)
    where
        T: Node,
    {
        // This can't update the bounding box, because the node's size isn't
        // known. You're on your own.
        self.group.append(node)
    }

    pub fn translate_to(&mut self, new_low: P2) -> Result<(), Error> {
        let bound = self.bound().ok_or_else(|| {
            format_err!(
                "Cannot translate group to a position because current bound is not known"
            )
        })?;

        let new_low = new_low * SCALE;
        let trans_vec = new_low - bound.low;

        self.translation = if let Some(current) = self.translation {
            Some(current + trans_vec)
        } else {
            Some(trans_vec)
        };
        Ok(())
    }
}

impl ToSvg for SvgGroup {
    type Output = Group;

    fn finalize(self) -> Self::Output {
        let mut group = self.group;
        if let Some(trans_vec) = self.translation {
            group.assign(
                "transform",
                format!("translate({},{})", trans_vec.x, trans_vec.y),
            );
        }
        group
    }
}

impl Bounded for SvgGroup {
    fn bound(&self) -> Option<Bound> {
        if let Some(bound) = self.bound {
            if let Some(trans_vec) = self.translation {
                Some(bound.translate(trans_vec))
            } else {
                Some(bound)
            }
        } else {
            None
        }
    }
}

impl Bound {
    pub fn new() -> Bound {
        Bound::from_origin(0., 0.)
    }

    pub fn empty_at(pos: P2) -> Bound {
        Bound {
            low: pos * SCALE,
            high: pos * SCALE,
        }
    }

    fn from_origin(width: f32, height: f32) -> Bound {
        Bound {
            low: P2::origin(),
            high: P2::new(width * SCALE, height * SCALE),
        }
    }

    fn view_box(&self) -> (f32, f32, f32, f32) {
        (self.low.x, self.low.y, self.width(), self.height())
    }

    pub fn width(&self) -> f32 {
        self.high.x - self.low.x
    }

    pub fn height(&self) -> f32 {
        self.high.y - self.low.y
    }

    fn size(&self) -> V2 {
        V2::new(self.width(), self.height())
    }

    pub fn center(&self) -> P2 {
        self.relative_pos(0.5, 0.5)
    }

    pub fn relative_pos(
        &self,
        width_fraction: f32,
        height_fraction: f32,
    ) -> P2 {
        let offset = V2::new(
            width_fraction * self.width(),
            height_fraction * self.height(),
        );
        self.low + offset
    }

    fn translate(&self, trans_vec: V2) -> Bound {
        Bound {
            low: self.low + trans_vec,
            high: self.high + trans_vec,
        }
    }

    pub fn union(&self, other: Option<Bound>) -> Bound {
        if let Some(other) = other {
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
        } else {
            *self
        }
    }

    pub fn intersect(&self, other: Option<Bound>) -> Option<Bound> {
        if let Some(other) = other {
            let low = P2::new(
                self.low.x.max(other.low.x),
                self.low.y.max(other.low.y),
            );
            let high = P2::new(
                self.high.x.min(other.high.x),
                self.high.y.min(other.high.y),
            );

            if low.x < high.x && low.y < high.y {
                Some(Bound {
                    low: low,
                    high: high,
                })
            } else {
                None
            }
        } else {
            Some(*self)
        }
    }

    pub fn union_all(bounds: &[Option<Bound>]) -> Bound {
        // ignore missing bounds (None), unwrap the rest
        let mut bounds = bounds.into_iter().skip_while(|b| b.is_none());
        let mut answer: Bound =
            bounds.next().expect("no bounds to union").unwrap();
        for b in bounds {
            answer = answer.union(*b);
        }
        answer
    }

    pub fn intersect_all(bounds: &[Option<Bound>]) -> Option<Bound> {
        let mut bounds = bounds.into_iter().skip_while(|b| b.is_none());
        let mut answer: Bound =
            bounds.next().expect("no bounds to intersect").unwrap();
        for b in bounds {
            answer = answer.intersect(*b)?;
        }
        Some(answer)
    }

    pub fn contains(&self, other: &Bound) -> bool {
        self.low.x <= other.low.x && self.low.y <= other.low.y
            && self.high.x >= other.high.x
            && self.high.y >= other.high.y
    }
}

impl SvgText {
    fn line_height(&self) -> f32 {
        // self.size * 1.1
        self.size
    }

    fn total_height(&self) -> f32 {
        self.line_height() * ((self.lines.len() as f32) - 1.)
    }
}

impl ToSvg for SvgText {
    type Output = Group;

    fn finalize(self) -> Self::Output {
        let mut g = Group::new();
        let mut y = self.pos.y - self.total_height() / 2.;
        let line_height = self.line_height();
        for line in self.lines {
            let text = Text::new()
            .set("x", self.pos.x)
            .set("y", y)
            .set("font-size", self.size)
            .set("font-style", "normal")
            .set("font-weight",  "bold")
            .set("font-family",  "sans-serif")
            .set("dominant-baseline", "central") // center vertically
            .set("text-anchor", "middle") // center horizontally
            .set("fill", self.color)
            .add(node::Text::new(line.to_owned()));

            g.append(text);
            y += line_height;
        }
        g
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
