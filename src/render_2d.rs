use error::LapstrakeError;
use scad_dots::core::MinMaxCoord;
use scad_dots::utils::{Axis, P2, V2};

use svg::node::element::path::Data;
use svg::node::element::{Circle, Group, Path, Rectangle, Text};
use svg::node::Value;
use svg::{self, node, Document, Node};

/// The PPI is not entirely standardized between svg rendering programs.
/// Inkscape currently use 96, but Inkscape version 0.91 and before used 90. In
/// Illustrator, it's adjustable. If the svg program assumes a different PPI
/// than what is used here, the scale will be wrong. Scale bars are a good
/// safety feature.
const PIXELS_PER_INCH: f32 = 96.;

pub struct SvgDoc {
    contents: SvgGroup,
}

#[derive(Clone)]
pub struct SvgGroup {
    contents: Vec<Box<ToSvg>>,
    bound: Option<Bound>,
    translation: Option<V2>,
}

/// Example:
///
/// ```
/// extern crate lapstrake;
/// extern crate scad_dots;
/// use scad_dots::utils::P2;
/// use lapstrake::render_2d::{SvgPath, PathStyle2, SvgColor};
/// let path = SvgPath::new(vec![P2::new(0., 0.), P2::new(50., 50.), P2::new(100., 20.)])
///     .style(PathStyle2::LineWithDots)
///     .stroke(SvgColor::Blue, 2.);
/// let scale = 1./12.;
/// path.save("tests/tmp/bluepath.svg", scale).unwrap();
/// ```
#[derive(Clone, Debug)]
pub struct SvgPath {
    points: Vec<P2>,
    stroke: Stroke,
    style: PathStyle2,
    is_closed: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum PathStyle2 {
    Dots,
    Line,
    LineWithDots,
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

#[derive(Clone, Debug)]
pub struct SvgText {
    pub lines: Vec<String>,
    pub pos: P2,
    pub color: SvgColor,
    pub size: f32,
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

#[derive(Clone, Copy, Debug)]
pub struct Bound {
    pub low: P2,
    pub high: P2,
}

pub trait Bounded {
    fn bound(&self) -> Option<Bound>;
}

pub trait ToSvg: 'static + CloneToSvg {
    fn finalize_to(&self, group: &mut Group, scale_from_feet: f32);
}

#[doc(hidden)]
pub trait CloneToSvg {
    fn clone(&self) -> Box<ToSvg>;
}

impl<T> CloneToSvg for T
where
    T: ToSvg + Clone,
{
    fn clone(&self) -> Box<ToSvg> {
        Box::new(Clone::clone(self))
    }
}

impl Clone for Box<ToSvg> {
    fn clone(&self) -> Self {
        CloneToSvg::clone(&**self)
    }
}

impl SvgDoc {
    pub fn new() -> SvgDoc {
        SvgDoc {
            contents: SvgGroup::new(),
        }
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

    pub fn save(
        self,
        filename: &str,
        scale_from_feet: f32,
    ) -> Result<(), LapstrakeError> {
        println!("Saving svg file {}.", filename);
        Ok(svg::save(filename, &self.finalize(scale_from_feet))?)
    }

    fn finalize(self, scale_from_feet: f32) -> Document {
        let mut doc = Document::new();
        let mut group = Group::new();
        if let Some(bound) = self.bound() {
            let background =
                SvgRect::new(bound.low, bound.size()).fill(SvgColor::White);
            background.finalize_to(&mut group, scale_from_feet);
            // doc.append(background);
            doc.assign("viewBox", bound.view_box(scale_from_feet));
        }
        self.contents.finalize_to(&mut group, scale_from_feet);
        doc.append(group);
        doc
    }
}

impl Bounded for SvgDoc {
    fn bound(&self) -> Option<Bound> {
        self.contents.bound
    }
}

impl SvgGroup {
    pub fn new() -> SvgGroup {
        SvgGroup {
            contents: Vec::new(),
            bound: None,
            translation: None,
        }
    }

    pub fn new_grid(
        contents: Vec<SvgGroup>,
        spacing: f32,
    ) -> Result<SvgGroup, LapstrakeError> {
        let num_rows = (contents.len() as f32).sqrt() as usize;
        SvgGroup::grid_helper(contents, spacing, num_rows)
    }

    pub fn new_vertical(
        contents: Vec<SvgGroup>,
        spacing: f32,
    ) -> Result<SvgGroup, LapstrakeError> {
        let num_rows = contents.len();
        SvgGroup::grid_helper(contents, spacing, num_rows)
    }

    pub fn new_horizontal(
        contents: Vec<SvgGroup>,
        spacing: f32,
    ) -> Result<SvgGroup, LapstrakeError> {
        SvgGroup::grid_helper(contents, spacing, 1)
    }

    fn grid_helper(
        contents: Vec<SvgGroup>,
        spacing: f32,
        num_rows: usize,
    ) -> Result<SvgGroup, LapstrakeError> {
        let mut group = SvgGroup::new();
        let mut column_bound = Bound::new();
        let y_spacing = V2::new(0., spacing);
        let x_spacing = V2::new(spacing, 0.);
        for (i, mut sub_group) in contents.into_iter().enumerate() {
            sub_group
                .translate_to(column_bound.relative_pos(0., 1.) + y_spacing)?;
            column_bound = column_bound.union(sub_group.bound());
            group.append(sub_group);

            if i % num_rows == num_rows - 1 {
                column_bound = Bound::empty_at(
                    column_bound.relative_pos(1., 0.) + x_spacing,
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

        // thing.finalize_to(&mut self.contents);
        self.contents.push(Box::new(thing));
    }

    pub fn translate_to(&mut self, new_low: P2) -> Result<(), LapstrakeError> {
        let bound = self.bound().ok_or_else(|| {
            LapstrakeError::Draw.context(
                "Cannot translate group to a position because current bound is not known"
            )
        })?;

        let trans_vec = new_low - bound.low;

        self.translation = if let Some(current) = self.translation {
            Some(current + trans_vec)
        } else {
            Some(trans_vec)
        };
        Ok(())
    }

    fn finalize(&self, scale_from_feet: f32) -> Group {
        let scale = scale(scale_from_feet);

        let mut group = Group::new();
        for item in &self.contents {
            item.finalize_to(&mut group, scale_from_feet);
        }
        if let Some(trans_vec) = self.translation {
            group.assign(
                "transform",
                format!(
                    "translate({},{})",
                    trans_vec.x * scale,
                    trans_vec.y * scale
                ),
            );
        }
        group
    }
}

impl ToSvg for SvgGroup {
    fn finalize_to(&self, group: &mut Group, scale_from_feet: f32) {
        group.append(self.finalize(scale_from_feet));
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

impl SvgPath {
    pub fn new(points: Vec<P2>) -> SvgPath {
        SvgPath {
            points: points,
            stroke: Stroke {
                color: SvgColor::Black,
                width: 1.,
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
            width: width,
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
        self.points.extend(new_points)
    }

    pub fn save(
        self,
        filename: &str,
        scale_from_feet: f32,
    ) -> Result<(), LapstrakeError> {
        let mut doc = SvgDoc::new();
        doc.append(self);
        doc.save(filename, scale_from_feet)?;
        Ok(())
    }

    fn dots(&self) -> SvgGroup {
        let radius = self.stroke.width;
        let color = self.stroke.color;

        let mut group = SvgGroup::new();
        for p in &self.points {
            group.append(SvgCircle::new(p.to_owned(), radius).fill(color));
        }
        group
    }

    fn path_data(&self, scale_from_feet: f32) -> Data {
        let scale = scale(scale_from_feet);
        let mut data = Data::new();
        let mut points = self.points.iter().map(|p| p * scale);
        let first = points.next().expect("path is empty");
        data = data.move_to(to_tuple(&first));
        for p in points {
            data = data.line_to(to_tuple(&p));
        }
        if self.is_closed {
            data = data.close();
        }
        data
    }
}

impl ToSvg for SvgPath {
    fn finalize_to(&self, group: &mut Group, scale_from_feet: f32) {
        let scale = scale(scale_from_feet);
        if self.style.has_line() {
            let mut path = Path::new();
            path.assign("d", self.path_data(scale_from_feet));
            path.assign("stroke", self.stroke.color);
            path.assign("stroke-width", self.stroke.width * scale);
            path.assign("fill", "none");
            group.append(path);
        }

        if self.style.has_dots() {
            group.append(self.dots().finalize(scale_from_feet));
        }
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
            pos: pos,
            radius: radius,
            stroke: None,
            fill: None,
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
}

impl ToSvg for SvgCircle {
    fn finalize_to(&self, group: &mut Group, scale_from_feet: f32) {
        let scale = scale(scale_from_feet);
        let mut element = Circle::new()
            .set("cx", self.pos.x * scale)
            .set("cy", self.pos.y * scale)
            .set("r", self.radius * scale);

        if let Some(stroke) = self.stroke {
            element.assign("stroke", stroke.color);
            element.assign("stroke-width", stroke.width * scale);
        }

        if let Some(color) = self.fill {
            element.assign("fill", color);
        } else {
            element.assign("fill", "none");
        }

        group.append(element);
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
            pos: pos,
            size: size,
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
    fn finalize_to(&self, group: &mut Group, scale_from_feet: f32) {
        let scale = scale(scale_from_feet);
        let mut element = Rectangle::new()
            .set("x", self.pos.x * scale)
            .set("y", self.pos.y * scale)
            .set("width", self.size.x * scale)
            .set("height", self.size.y * scale);

        if let Some(stroke) = self.stroke {
            element.assign("stroke", stroke.color);
            element.assign("stroke-width", stroke.width * scale);
        }

        if let Some(color) = self.fill {
            element.assign("fill", color);
        } else {
            element.assign("fill", "none");
        }

        if let Some(fillet) = self.fillet {
            element.assign("rx", fillet.x * scale);
            element.assign("ry", fillet.y * scale);
        }
        group.append(element);
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

impl SvgText {
    fn line_height(&self) -> f32 {
        self.size
    }

    fn total_height(&self) -> f32 {
        self.line_height() * ((self.lines.len() as f32) - 1.)
    }
}

impl ToSvg for SvgText {
    fn finalize_to(&self, group: &mut Group, scale_from_feet: f32) {
        let scale = scale(scale_from_feet);
        let mut y = (self.pos.y - self.total_height() / 2.) * scale;
        let line_height = self.line_height() * scale;
        for line in &self.lines {
            let text = Text::new()
                .set("x", self.pos.x * scale)
            .set("y", y)
            .set("font-size", self.size * scale)
            .set("font-style", "normal")
            .set("font-weight",  "bold")
            .set("font-family",  "sans-serif")
            .set("dominant-baseline", "central") // center vertically
            .set("text-anchor", "middle") // center horizontally
            .set("fill", self.color)
            .add(node::Text::new(line.to_owned()));

            group.append(text);
            y += line_height;
        }
    }
}

impl Bounded for SvgText {
    fn bound(&self) -> Option<Bound> {
        // We don't know how big text is, because rendering it is complicated :(
        None
    }
}
impl Bound {
    pub fn new() -> Bound {
        Bound::from_origin(0., 0.)
    }

    pub fn empty_at(pos: P2) -> Bound {
        Bound {
            low: pos,
            high: pos,
        }
    }

    fn from_origin(width: f32, height: f32) -> Bound {
        Bound {
            low: P2::origin(),
            high: P2::new(width, height),
        }
    }

    fn view_box(&self, scale_from_feet: f32) -> (f32, f32, f32, f32) {
        let scale = scale(scale_from_feet);
        (
            self.low.x * scale,
            self.low.y * scale,
            self.width() * scale,
            self.height() * scale,
        )
    }

    pub fn width(&self) -> f32 {
        self.high.x - self.low.x
    }

    pub fn height(&self) -> f32 {
        self.high.y - self.low.y
    }

    pub fn size(&self) -> V2 {
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
        self.low.x <= other.low.x
            && self.low.y <= other.low.y
            && self.high.x >= other.high.x
            && self.high.y >= other.high.y
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

fn scale(scale_from_feet: f32) -> f32 {
    scale_from_feet * 12. * PIXELS_PER_INCH
}

pub fn make_scale_bar() -> Result<SvgGroup, LapstrakeError> {
    let stroke = 0.05;
    let short_length = 1.;
    let long_length = 10.;
    let cap_length = short_length / 10.;
    let font_size = short_length / 4.;

    let short = make_capped_line(short_length - stroke, cap_length)
        .stroke(SvgColor::Black, stroke);

    let long = make_capped_line(long_length - stroke, cap_length)
        .stroke(SvgColor::Black, stroke);

    let short_label = SvgText {
        lines: vec!["1 ft".into()],
        pos: short.bound().unwrap().center() + V2::new(0., font_size),
        color: SvgColor::Black,
        size: font_size,
    };
    let long_label = SvgText {
        lines: vec!["10 ft".into()],
        pos: long.bound().unwrap().center() + V2::new(0., font_size),
        color: SvgColor::Black,
        size: font_size,
    };

    let mut short_group = SvgGroup::new();
    let mut long_group = SvgGroup::new();
    short_group.append(short);
    short_group.append(short_label);
    long_group.append(long);
    long_group.append(long_label);
    SvgGroup::new_vertical(vec![long_group, short_group], cap_length)
}

fn make_capped_line(length: f32, cap_length: f32) -> SvgPath {
    let pos = P2::origin();
    let cap_offset = V2::new(0., cap_length);
    let line_offset = V2::new(length, 0.);

    SvgPath::new(vec![
        pos + cap_offset,
        pos,
        pos + line_offset,
        pos + line_offset + cap_offset,
    ])
}
