use std::io::{self, BufRead, Write};

mod canvas;
mod data;
mod frame;

use canvas::{Canvas, Mode};
use data::{Data, MARKS};
use frame::{Frame, PAD};
use hdrhistogram::Histogram;

fn main() -> io::Result<()> {
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();

    let log_x = false;
    let log_y = false;
    let x_is_row = true;
    let width = 90;
    let height = 25;
    let mut mode = Mode::Count;
    let cdf = false;
    if cdf {
        assert!(x_is_row);
        assert!(!log_x);
        // log y is interpreted as log of the _input_ not _output_
        mode = Mode::Count;
    }

    let mut data = Data::default();
    let mut canvas = Canvas::new(height, width, mode);

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
                if column + 1 > data.ys.len() {
                    if column >= MARKS.len() {
                        // we can't label these ones!
                        continue;
                    }
                    assert_eq!(
                        column,
                        data.ys.len(),
                        "we will only ever add a single new column at a time"
                    );
                    // discovered a new column!
                    // need to add the column, which means adding empty
                    // values for that column for all pre-existing rows.
                    data.ys.push(vec![f64::NAN; data.xs.len()]);
                }
                data.ys[column].push(v);
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
            data.xs.push(x);
        }

        // make sure we fill in the other column values
        for y in &mut data.ys {
            if y.len() < data.xs.len() {
                assert_eq!(y.len(), data.xs.len() - 1);
                y.push(f64::NAN);
            }
        }
    }

    if log_x {
        for x in &mut data.xs {
            if *x != 0. {
                *x = x.log10();
            }
        }
    }
    if log_y {
        for y in data.ys.iter_mut().flatten() {
            if *y != 0. {
                *y = y.log10();
            }
        }
    }

    let mut frame = Frame::new_over(width, height, &data);
    let (min_y, _) = frame.y_bounds();
    let (_, range_y) = frame.range_xy();

    // apply transformations
    if cdf {
        data.xs.clear();

        let plot_width = (width - PAD) as f64;
        for ys in &mut data.ys {
            let mut histogram =
                Histogram::<u32>::new_with_bounds(1, width as u64, 3).expect("3 is a valid sigfig");
            for y in ys.drain(..) {
                let y_as_fraction_of_axis = (y - min_y) / range_y;
                let y_as_future_column = (plot_width * y_as_fraction_of_axis).round() as u64;

                histogram
                    .record(y_as_future_column)
                    .expect("value is in range");
            }

            for (i, bin) in histogram.iter_linear(1).enumerate() {
                let x_as_column = bin.value_iterated_to() as f64;
                let x = min_y + (x_as_column / plot_width) * range_y;
                if i >= data.xs.len() {
                    data.xs.push(x);
                } else {
                    assert_eq!(x, data.xs[i]);
                }
                ys.push(bin.percentile());
            }
        }

        for y in &mut data.ys {
            y.resize(data.xs.len(), y.last().copied().unwrap_or(f64::NAN));
        }

        frame = Frame::new_over(width, height, &data);
    }

    frame.draw_into(&mut canvas);
    data.draw_into(&mut canvas, &frame);

    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    let (min_x, max_x) = frame.x_bounds();
    let (min_y, max_y) = frame.y_bounds();
    if log_x {
        write!(stdout, "    log x: [{min_x} - {max_x}]")?;
    } else {
        write!(stdout, "    x: [{min_x} - {max_x}]")?;
    }
    if log_y {
        write!(stdout, "    log y: [{min_y} - {max_y}]")?;
    } else {
        write!(stdout, "    y: [{min_y} - {max_y}]")?;
    }
    if let Mode::Dot = canvas.mode {
        write!(stdout, " -- ")?;
        for column in 0..data.ys.len() {
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
    writeln!(stdout, "{canvas}")?;

    Ok(())
}
