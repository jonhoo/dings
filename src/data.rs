use crate::{Canvas, Frame, Mode};

pub const MARKS: &[u8] = b"@*^!~%ABCDEFGHIJKLMNOPQRSTUVWXYZ";

#[derive(Debug, Default)]
pub(crate) struct Data {
    pub(crate) flipped: bool,
    pub(crate) xs: Vec<f64>,
    pub(crate) ys: Vec<Vec<f64>>,
}

type MinMaxValsFn = (Option<f64>, Option<f64>, Option<f64>, Option<f64>);

impl Data {
    pub(crate) fn flip(&mut self) {
        self.flipped = !self.flipped;
    }
    pub(crate) fn get_min_max_vals(&self) -> MinMaxValsFn {
        if self.flipped {
            let (min_x, max_x) = get_min_max_vec_vec(&self.ys);
            let (min_y, max_y) = get_min_max_vec(&self.xs);
            (min_x, max_x, min_y, max_y)
        } else {
            let (min_x, max_x) = get_min_max_vec(&self.xs);
            let (min_y, max_y) = get_min_max_vec_vec(&self.ys);
            (min_x, max_x, min_y, max_y)
        }
    }
    pub(crate) fn draw_into(&self, canvas: &mut Canvas, using: &Frame) {
        for (row, x) in self.xs.iter().copied().enumerate() {
            let x_cell: usize;
            if !self.flipped {
                x_cell = using.x_to_column(x);
            } else {
                x_cell = using.y_to_row(x);
            }
            for (column, ys) in self.ys.iter().enumerate() {
                let y = ys[row];

                if !y.is_finite() {
                    continue;
                }
                const CMP_PAD: f64 = 0.001;
                let y_cell: usize;
                let cell: &mut u8;
                let mode = canvas.mode;
                if !self.flipped {
                    let (min_x, max_x) = using.x_bounds();
                    assert!(x >= min_x - CMP_PAD);
                    assert!(x <= max_x + CMP_PAD);

                    y_cell = using.y_to_row(y);
                    cell = canvas
                        .cell(y_cell, x_cell)
                        .expect("invalid cell ({y_cell}, {x_cell}) for data point ({x}, {y})");
                } else {
                    let (min_y, max_y) = using.x_bounds();
                    assert!(y >= min_y - CMP_PAD);
                    assert!(y <= max_y + CMP_PAD);

                    y_cell = using.x_to_column(y);

                    cell = canvas
                        .cell(x_cell, y_cell)
                        .expect("invalid cell ({y_cell}, {x_cell}) for data point ({x}, {y})");
                };

                match mode {
                    Mode::Dot => *cell = MARKS[column],
                    Mode::Count => {
                        // in count mode, we want each cell to display the number of points that fall
                        // within that cell from _any_ dataset. we can get this behaviour by abusing
                        // the `u8` that gets stored for every cell. we simply use that `u8` as a
                        // counter (well, counter in base36...) that saturates in '#'.
                        *cell = match *cell {
                            // NOTE: it's intentional that we _don't_ match 'z' here
                            #[allow(clippy::almost_complete_range)]
                            b'0'..b'9' | b'a'..b'z' => *cell + 1,
                            b'9' => b'a',
                            b'z' | b'#' => b'#',
                            // this part is (extra) cursed.
                            // something needs to initialize the u8 base36 counters (sorry not sorry),
                            // because their previous value could be ' ' from the blank canvas, or '+',
                            // '-', '.', or '|' from the axes. we _could_ do that with a loop before
                            // this one that sets ever data point cell to '0', but doing so would mean
                            // we also need to compute all the cell values multiple times (or cache
                            // them somehow). instead, we simply assume that any non-base36-and-not-#
                            // value is 0.
                            //
                            // _but_, we want non-overlapping values to keep their mark so different
                            // datasets can be told apart, so 1 is MARKS[column]. this in turn requires
                            // that none of the axis marks are in MARKS.
                            b'-' | b'+' | b'.' | b'|' | b' ' => MARKS[column],
                            c if MARKS.contains(&c) => b'2',
                            _ => unreachable!(
                                "cell at ({y_cell}, {x_cell}) held unexpected counting mark '{cell}'"
                            ),
                        }
                    }
                }
            }
        }
    }
}

fn get_min_max_vec(vecf64: &Vec<f64>) -> (Option<f64>, Option<f64>) {
    let elems = vecf64.iter().filter(|v| v.is_finite());
    (
        elems.clone().copied().min_by(f64::total_cmp),
        elems.copied().max_by(f64::total_cmp),
    )
}

fn get_min_max_vec_vec(vecvecf64: &Vec<Vec<f64>>) -> (Option<f64>, Option<f64>) {
    let elems = vecvecf64.iter().flatten().filter(|v| v.is_finite());
    (
        elems.clone().copied().min_by(f64::total_cmp),
        elems.copied().max_by(f64::total_cmp),
    )
}
