use std::io::{self, BufRead, Write};

const MARKS: &[u8] = b"@*^!~%ABCDEFGHIJKLMNOPQRSTUVWXYZ";

#[derive(Debug)]
struct Canvas {
    stride: usize,
    cells: Box<[u8]>,
}

impl Canvas {
    fn new(rows: usize, columns: usize) -> Self {
        Self {
            stride: columns,
            cells: vec![b' '; rows * columns].into_boxed_slice(),
        }
    }

    fn rows(&self) -> impl Iterator<Item = &[u8]> {
        self.cells.chunks_exact(self.stride)
    }

    fn cell(&mut self, row: usize, column: usize) -> Option<&mut u8> {
        self.cells.get_mut(row * self.stride + column)
    }
}

fn main() -> io::Result<()> {
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();

    let x_is_row = true;
    let width = 90;
    let height = 25;
    enum Mode {
        Dot,
        Count,
    }
    let mut mode = Mode::Dot;
    let cdf = false;
    if cdf {
        mode = Mode::Count;
    }

    let mut xs: Vec<f64> = Vec::new();
    let mut ys: Vec<Vec<f64>> = Vec::new();
    let mut canvas = Canvas::new(height, width);

    let mut line = String::new();
    for row in 0.. {
        line.clear();
        let mut x = x_is_row.then_some(row as f64);
        let n = stdin.read_line(&mut line)?;
        if n == 0 {
            break;
        }

        let mut column = 0;
        let mut line = line.trim_end();
        while !line.is_empty() {
            let (next_num, remainder) = line
                .split_once(|c| !matches!(c, '0'..='9' | '-' | '+' | '.'))
                .unwrap_or((line, ""));
            line = remainder;
            let v = match next_num.parse::<f64>() {
                Ok(v) => {
                    assert!(!v.is_infinite(), "{v}");
                    v
                }
                Err(_) => {
                    // invalid values are treated as missing
                    f64::NAN
                }
            };
            if x.is_some() {
                // have a data point!
                if column + 1 > ys.len() {
                    if column >= MARKS.len() {
                        // we can't label these ones!
                        continue;
                    }
                    assert_eq!(
                        column,
                        ys.len(),
                        "we will only ever add a single new column at a time"
                    );
                    // discovered a new column!
                    // need to add the column, which means adding empty
                    // values for that column for all pre-existing rows.
                    ys.push(vec![f64::NAN; xs.len()]);
                }
                ys[column].push(v);
                column += 1;
            } else {
                // found x value
                x = Some(v);
            }
        }

        // whatever x value we discovered is the x for the row
        // NOTE: if this is None, that means there were no column values at all, which is
        // equivalent to an empty line, which we simply don't count as a sample. note also that
        // this means ys has not been pushed to either.
        if let Some(x) = x {
            xs.push(x);
        }

        // make sure we fill in the other column values
        for y in &mut ys {
            if y.len() < xs.len() {
                assert_eq!(y.len(), xs.len() - 1);
                y.push(f64::NAN);
            }
        }
    }

    let min_x = xs
        .iter()
        .filter(|v| v.is_finite())
        .copied()
        .min_by(f64::total_cmp);
    let max_x = xs
        .iter()
        .filter(|v| v.is_finite())
        .copied()
        .max_by(f64::total_cmp);
    let min_y = ys
        .iter()
        .flatten()
        .filter(|v| v.is_finite())
        .copied()
        .min_by(f64::total_cmp);
    let max_y = ys
        .iter()
        .flatten()
        .filter(|v| v.is_finite())
        .copied()
        .max_by(f64::total_cmp);

    // apply transformations
    if cdf {
        for (column, ys) in ys.iter().enumerate() {
            // let mut histogram = Histogram::new_with_bounds(5).expect("5 is a valid sigfig");
        }
    }

    let (Some(mut min_x), Some(max_x)) = (min_x, max_x) else {
        return Ok(());
    };
    let (Some(mut min_y), Some(max_y)) = (min_y, max_y) else {
        return Ok(());
    };

    /* Override bounds that would lead to a range of zero, to avoid a
     * crash when plotting. (Found by afl.) */
    let mut max_x = f64::max(min_x + 1.0, max_x);
    let mut max_y = f64::max(min_y + 1.0, max_y);

    let mut range_x = max_x - min_x;
    let mut range_y = max_y - min_y;

    // If along a given axis, the data doesn't intersect the axis itself, we'd like to start/end
    // plotting at the axis. this makes datasets read slightly more reasonably since they'll "start
    // at 0", rather than starting at some other value that just happens to be the minimum of the
    // distribution. however, if the data is sufficiently far from 0, and has a sufficiently small
    // range, plotting from zero would "squish" it so much that it won't be readable, and so we try
    // to find a heuristic in between that generally results in reasonable display behaviour.
    const CROSS_PAD: f64 = 2.0;
    let crosses_x = min_x <= 0. && max_x >= 0.;
    let crosses_y = min_y <= 0. && max_y >= 0.;
    if !crosses_x {
        if min_x > 0. && (min_x - range_x * CROSS_PAD) < 0. {
            min_x = 0.;
            range_x = max_x;
        } else if max_x < 0. && (max_x + range_x * CROSS_PAD) > 0. {
            max_x = 0.;
            range_x = -min_x;
        }
    }
    if !crosses_y {
        if min_y > 0. && (min_y - range_y * CROSS_PAD) < 0. {
            min_y = 0.;
            range_y = max_y;
        } else if max_y < 0. && (max_y + range_y * CROSS_PAD) > 0. {
            max_y = 0.;
            range_y = -min_y;
        }
    }

    let pad = 2;
    let plot_width = (width - pad) as f64;
    let plot_height = (height - pad) as f64;

    // compute origin
    let y0_is_visible = min_y <= 0. && max_y >= 0.;
    let x0_is_visible = min_x <= 0. && max_x >= 0.;

    let mut draw_vertical_at_x = 0.;
    let mut draw_horizontal_at_y = 0.;
    if !x0_is_visible {
        if min_x > 0. {
            draw_vertical_at_x = min_x;
        } else {
            draw_vertical_at_x = max_x;
        }
    }
    if !y0_is_visible {
        if min_y > 0. {
            draw_horizontal_at_y = min_y;
        } else {
            draw_horizontal_at_y = max_y;
        }
    }

    let hat_as_fraction_of_axis = (draw_horizontal_at_y - min_y) / range_y;
    let draw_horizontal_at_row = (plot_height * hat_as_fraction_of_axis).round() as usize;
    let draw_horizontal_at_row = height - draw_horizontal_at_row - 1;

    let vat_as_fraction_of_axis = (draw_vertical_at_x - min_x) / range_x;
    let draw_vertical_at_column = (plot_width * vat_as_fraction_of_axis).round() as usize;

    // draw in the axes
    // draw the vertical (Y) axis (so where X = 0)
    for row in 0..height {
        let c = if x0_is_visible {
            if row % 5 == 0 {
                b'+'
            } else {
                b'|'
            }
        } else {
            if row % 5 == 0 {
                b'.'
            } else {
                b' '
            }
        };
        let Some(cell) = canvas.cell(row, draw_vertical_at_column) else {
            panic!("invalid cell ({row}, {draw_vertical_at_column}) for axis component ({draw_vertical_at_x}, _)");
        };
        *cell = c;
    }
    // draw the horizontal (X) axis (so where Y = 0)
    for column in 0..width {
        let c = if y0_is_visible {
            if column % 5 == 0 {
                b'+'
            } else {
                b'-'
            }
        } else {
            if column % 5 == 0 {
                b'.'
            } else {
                b' '
            }
        };
        let Some(cell) = canvas.cell(draw_horizontal_at_row, column) else {
            panic!("invalid cell ({draw_horizontal_at_row}, {column}) for axis component ({draw_horizontal_at_y}, _)");
        };
        *cell = c;
    }

    // where the axes meet, put a +
    let intersection = canvas
        .cell(draw_horizontal_at_row, draw_vertical_at_column)
        .expect("must have hit one of the panics above");
    *intersection = b'+';

    // draw in the points
    for (row, x) in xs.iter().copied().enumerate() {
        let x_as_fraction_of_axis = (x - min_x) / range_x;
        let x_cell = (plot_width * x_as_fraction_of_axis).round() as usize;

        for (column, ys) in ys.iter().enumerate() {
            let y = ys[row];

            const CMP_PAD: f64 = 0.001;
            assert!(x >= min_x - CMP_PAD);
            assert!(x <= max_x + CMP_PAD);

            let y_as_fraction_of_axis = (y - min_y) / range_y;
            let y_cell_from_top = (plot_height * y_as_fraction_of_axis).round() as usize;

            // flip y; 0 at bottom of plot
            let y_cell = height - y_cell_from_top - 1;

            let Some(cell) = canvas.cell(y_cell, x_cell) else {
                panic!("invalid cell ({y_cell}, {x_cell}) for data point ({x}, {y})");
            };

            match mode {
                Mode::Dot => *cell = MARKS[column],
                Mode::Count => {
                    // in count mode, we want each cell to display the number of points that fall
                    // within that cell from _any_ dataset. we can get this behaviour by abusing
                    // the `u8` that gets stored for every cell. we simply use that `u8` as a
                    // counter (well, counter in base36...) that saturates in '#'.
                    *cell = match *cell {
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

    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    write!(
        stdout,
        "    x: [{min_x} - {max_x}]    y: [{min_y} - {max_y}]"
    )?;
    if let Mode::Dot = mode {
        write!(stdout, " -- ")?;
        for column in 0..ys.len() {
            write!(
                stdout,
                "{}{}: {}",
                if column > 0 { ", " } else { "" },
                column,
                char::from(MARKS[column])
            )?;
        }
    }
    writeln!(stdout)?;
    for row in canvas.rows() {
        stdout.write_all(row)?;
        writeln!(stdout)?;
    }

    Ok(())
}
