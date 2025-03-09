use std::io::{self, BufRead, Write};

mod canvas;
mod data;
mod frame;

use canvas::{Canvas, Mode};
use data::{Data, MARKS};
use frame::Frame;

fn main() -> io::Result<()> {
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();

    let x_is_row = true;
    let width = 90;
    let height = 25;
    let mut mode = Mode::Dot;
    let cdf = false;
    if cdf {
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

    // apply transformations
    if cdf {
        for (column, ys) in data.ys.iter().enumerate() {
            // let mut histogram = Histogram::new_with_bounds(5).expect("5 is a valid sigfig");
        }
    }

    let frame = Frame::new_over(width, height, &data);
    frame.draw_into(&mut canvas);
    data.draw_into(&mut canvas, &frame);

    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    let (min_x, max_x) = frame.x_bounds();
    let (min_y, max_y) = frame.y_bounds();
    write!(
        stdout,
        "    x: [{min_x} - {max_x}]    y: [{min_y} - {max_y}]"
    )?;
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
