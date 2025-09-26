use args::Opt;
use canvas::{Canvas, Mode};
use data::{Data, MARKS};
use eyre::Context;
use frame::{Frame, PAD};
use hdrhistogram::Histogram;
use std::io::{BufRead, Write};

mod args;
mod canvas;
mod data;
mod frame;

fn main() -> eyre::Result<()> {
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();

    let Opt {
        log_x,
        log_y,
        x_is_row,
        width,
        height,
        mode,
        cdf,
        draw_axes,
        flip,
    } = Opt::parse_from_env().context("parse command-line arguments")?;

    let mut data = Data::default();
    let mut canvas = Canvas::new(height, width, mode);

    let mut line = String::new();
    for row in 0.. {
        line.clear();
        let mut x = x_is_row.then_some(row as f64);
        let n = stdin.read_line(&mut line).context("read input line")?;
        if n == 0 {
            break;
        }

        let mut column = 0;
        let mut line = line.trim_end();
        while !line.is_empty() {
            let (next_num, remainder) = line
                .split_once(|c| !matches!(c, '0'..='9' | '-' | '+' | '.' | 'E' | 'e'))
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

    //if -f flip, we flip xs and ys before any plotting.
    //EXAMPLE:
    //if we want to plot 1 2 3\n4 5 6\n, in normal mode, we would have:
    //data.xs=[0,1]
    //data.ys=[[1,4],[2,5],[3,6]].
    //This would produce the following series:
    //  @     *      ^
    //(0,1) (0,2) (0,3)
    //(1,4) (1,5) (1,6)
    //In order to preserve all the plotting mechanism, we want to obtain those new vecs:
    //data.xs=[1,4,2,5,3,6]
    //data.ys=[[0,1,N,N,N,N],[N,N,0,1,N,N],[N,N,N,N,0,1]] (N=NAN)
    //It would produce
    //  @     *      ^
    //(1,0) (2,0) (3,0)
    //(4,1) (5,1) (6,1)
    //
    if flip {
        let num_cols = data.ys.len();
        //for xs, we just have to flatten ys into a Vec
        let new_xs: Vec<f64> = data.ys.iter().flatten().copied().collect();

        //for ys, we have to adjust the len of each Vec to match new xs. We use Nan values as they
        //will be discard by the data.draw_into fn.
        if new_xs.len() > data.xs.len() {
            data.xs
                .extend(std::iter::repeat_n(f64::NAN, new_xs.len() - data.xs.len()));
        }
        //we repeat this vec the necessary number of times and we rotate values to the right.
        let mut new_ys: Vec<Vec<f64>> = std::iter::repeat_n(data.xs, num_cols).collect();
        for (index, elem) in new_ys.iter_mut().enumerate() {
            elem.rotate_right(2 * index);
        }

        (data.xs, data.ys) = (new_xs, new_ys);
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

    // if -A is passed, we don't draw axes.
    if draw_axes {
        frame.draw_into(&mut canvas);
    }
    data.draw_into(&mut canvas, &frame);

    let stdout = std::io::stdout();
    let stdout = stdout.lock();
    render(&data, &frame, &canvas, log_x, log_y, stdout).context("render output")?;

    Ok(())
}

fn render(
    data: &Data,
    frame: &Frame,
    canvas: &Canvas,
    log_x: bool,
    log_y: bool,
    mut out: impl Write,
) -> eyre::Result<()> {
    let (min_x, max_x) = frame.x_bounds();
    let (min_y, max_y) = frame.y_bounds();
    if log_x {
        write!(out, "    log x: [{min_x} - {max_x}]")?;
    } else {
        write!(out, "    x: [{min_x} - {max_x}]")?;
    }
    if log_y {
        write!(out, "    log y: [{min_y} - {max_y}]")?;
    } else {
        write!(out, "    y: [{min_y} - {max_y}]")?;
    }
    if let Mode::Dot = canvas.mode {
        write!(out, " -- ")?;
        #[allow(clippy::needless_range_loop)]
        for column in 0..data.ys.len() {
            write!(
                out,
                "{}{}: {}",
                if column > 0 { ", " } else { "" },
                column,
                char::from(MARKS[column])
            )?;
        }
    }
    writeln!(out)?;
    writeln!(out, "{canvas}")?;
    Ok(())
}
