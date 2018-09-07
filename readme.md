# Lapstrake

Lapstrake is a tool for model-ship builders. It takes a table of offsets
that describe the shape of a hull, and produces:

- A 3d rendering of the hull, using OpenSCAD.
- Half-breadth drawings. Coming soon: profile and waterline drawings.
- Cross-sections of the hull suitable for laser cutting, to make a
  frame for model-ship construction.
- Plank shapes that can be laser cut and then layed out to form the
  hull. It supports both lapstrake (a.k.a. clinker) construction, which has 
  overlap between planks, and carvel construction, which does not.


## Requirements

* [Rust](https://www.rust-lang.org/install.html)
* [OpenSCAD](http://www.openscad.org/) (optional: for viewing 3d renderings of the hull)

## Quick Start

To use the example ship data included in this repository:

1. Clone this repository.
2. From the `lapstrake` folder, run one of the following commands:
   - `cargo run -- wireframe` to view a 3d rendering of the hull in openSCAD.
   - `cargo run -- drawings` to save svg files of various diagrams
     of the hull (only half-breadths for now).
   - `cargo run -- stations` to save an svg file of station cross-section templates.
   - `cargo run -- planks` to make an svg file of plank templates. 
   - `cargo run -- help` for a complete list of commands and options.
   
## Customizing Ship Data

1. Starting with the [example](https://docs.google.com/spreadsheets/d/1VAPovAuHxfU8NDknkA-fIjc7hZLZ6ZLZ0003-x4P4KE/edit?usp=sharing) google spreadsheet as a template,
   fill in the "Data" sheet with the hull measurements of the of the ship you are
   interested in. All units must be entered as feet, inches, and
   eights of an inch separated by dashes. Measurements in the body of
   the table may be ommitted by writing "x". Diagonal lines are not
   currently supported. (You can add some, but they will be ignored.)
2. Fill out the "Config" sheet of the same google doc. [TODO: describe fields]. 
3. Fill out the "Planks" sheet. This lets you control where the top and bottom edges of each plank falls, on each station/cross-section (as fractions of the length of that cross-section of the hull). You can start with approximately equal plank widths and then go back later to adjust it for a smoother fit, after viewing the generated plank shapes.
4. Export each sheet as a separate csv file: `File > Download As >
   Comma-separated values (csv)`. Name them `data.csv`, `config.csv`, and `planks.csv`  repsectively, and save them in the `lapstrake/input` folder.
   
Because there is much variation in the format of offset tables in ship plans, the spreadsheet and the lapstrake program might require some modification to work with your data. Feel free to submit issues.


## Example Data

[Block Island Boat, H.I. Chapell, Cambridge Maryland, 1952](https://docs.google.com/spreadsheets/d/1VAPovAuHxfU8NDknkA-fIjc7hZLZ6ZLZ0003-x4P4KE/edit?usp=sharing)

## Todo

- Let the user set svg stroke width and color for laser cutting.
- Add profile and waterline drawings.
- Remove extra height grid lines from half-breadth drawing.
- Add 3d rendering of the complete hull as a shell.
- Document code more fully.
- Add pictures to readme
- Remove `number_of_planks` from the Config sheet, since it's implicitly in the Planks sheet now.
- add `cargo run -- poster` to make a poster out of the three diagrams, plus the table of hull measurements.
