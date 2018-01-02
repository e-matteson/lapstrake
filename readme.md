# Lapstrake

**Lapstrake is under construction. It is not done yet, and does not yet
 do the things this readme describes. This file is a roadmap of
 planned features.**

Lapstrake is a tool for ship afficionados. It takes a table of offsets
that describe the shape of a hull, and produces:

- A 3d rendering of the hull.
- Half-breadth, Profile, and Waterline drawings.
- Cross sections of the hull suitable for laser cutting, to make a
  frame for model ship construction.
- Planks that can be laser cut and then layed out to form the
  hull. Lapstrake supports both carvel construction, which has no
  overlap between planks, and lapstrake (a.k.a. clinker) construction,
  which has overlap.


## Instructions

1. Clone this repo.
2. Starting with the Google doc of example data below as a template,
   fill out the measurements of the hull of the ship you are
   interested in. All units must be entered as feet, inches, and
   eights of an inch separated by dashes. Measurements in the body of
   the table may be ommitted by writing "x". Diagonal lines are not
   currently supported. (You can add some, but they will be ignored.)
   Also fill out the "Config" sheet. [TODO: describe options].
3. Export the "Data" sheet as a csv file (File > Download As >
   Comma-separated values (csv)). Call it `data.csv` (lowercase), and
   save it in the toplevel "lapstrake" folder.
4. Also export the "Config" sheet as a csv file. Call it `config.csv`,
   and save it in the toplevel "lapstrake" folder.
5. In the "lapstrake" folder, run one of the following commands,
   depending on what you want:
   - `cargo run --bin render` to make a 3d rendering of the hull. This
     requires [openscad](http://www.openscad.org/) to be installed.
   - `cargo run --bin diagrams` to make svg files of various diagrams
     of the hull. They will be output as `half-breadth.svg`,
     `buttock.svg`, and `waterline.svg`.
   - `cargo run --bin poster` to make a poster out of the three
     diagrams, plus the table of hull measurements.
   - `cargo run --bin stations` to make an svg file of laser-cuttable
     stations. It will be output as `stations.svg`.
   - `cargo run --bin planks` to make an svg file of laser-cuttable
     planks. It will be output as `planks.svg`.


## Example Data

Example data for 
(https://docs.google.com/spreadsheets/d/1VAPovAuHxfU8NDknkA-fIjc7hZLZ6ZLZ0003-x4P4KE/edit?usp=sharing)
