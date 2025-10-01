use crate::canvas::Mode;
use eyre::Context;
use lexopt::prelude::*;

#[derive(Debug)]
pub(crate) struct Opt {
    pub(crate) log_x: bool,
    pub(crate) log_y: bool,
    pub(crate) x_is_row: bool,
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) mode: Mode,
    pub(crate) cdf: bool,
    pub(crate) draw_axes: bool,
}

impl Opt {
    pub fn parse_from_env() -> eyre::Result<Self> {
        let mut opt = Opt {
            log_x: false,
            log_y: false,
            x_is_row: true,
            width: 72,
            height: 40,
            mode: Mode::Dot,
            cdf: false,
            draw_axes: true,
        };
        let mut parser = lexopt::Parser::from_env();
        while let Some(arg) = parser.next().context("read next argument")? {
            match arg {
                Short('h') | Long("help") => {
                    cli_help();
                }
                Short('d') => {
                    let dim = parser.value().context("value for -d")?;
                    let Some(dim) = dim.to_str() else {
                        eyre::bail!("-d argument contains invalid characters");
                    };
                    if let Some((width, height)) = dim.split_once('x') {
                        opt.width = width.parse().context("parse width in -d argument")?;
                        opt.height = height.parse().context("parse height in -d argument")?;
                    } else {
                        eyre::bail!(
                            "-d must be specified as WxH (eg, 72x40, which is the default)"
                        );
                    }
                }
                Short('l') | Long("log") => {
                    let dim = parser.value().context("value for --log")?;
                    if dim == "x" {
                        opt.log_x = true;
                    } else if dim == "y" {
                        opt.log_y = true;
                    } else if dim == "c" {
                        eyre::bail!("--log c is not yet supported");
                    } else {
                        eyre::bail!("--log takes x, y, or c");
                    }
                }
                Short('m') | Long("mode") => {
                    let mode = parser.value().context("value for --mode")?;
                    if mode == "dot" {
                        opt.mode = Mode::Dot;
                    } else if mode == "count" {
                        opt.mode = Mode::Count;
                    } else {
                        eyre::bail!("--mode takes dot (the default) or count");
                    }
                }
                Short('x') => {
                    opt.x_is_row = false;
                }
                Long("cdf") => {
                    opt.cdf = true;
                }
                Short('A') => {
                    opt.draw_axes = false;
                }
                arg => return Err(arg.unexpected().into()),
            }
        }

        if opt.cdf {
            eyre::ensure!(
                opt.x_is_row,
                "CDF is only over the Y value; an explicit X value will be ignored"
            );
            eyre::ensure!(
            !opt.log_x,
            "CDF is only over the Y value and changes the axes; logarithmic X would have no effet"
        );
            // NOTE: log y is interpreted as log of the _input_ not _output_
        }

        Ok(opt)
    }
}

fn cli_help() {
    let title_color = 0;
    let color = 32;
    let commands_color = 32;
    println!(
        "\x1b[{}m Dings: a quick command-line data visualization tool.\x1b[0m",
        title_color
    );
    println!();
    println!(
        "\x1b[{};1m Usage:\x1b[0m\x1b[1m dings [-A] [-d WxH] [-f] [-h|--help] [-l|--log XY]
              [-m|--mode MODE] [--cdf] [-x] [FILE]\x1b[0m",
        color
    );

    let commands = [
        ("A", "don't draw axes"),
        ("d", "set width & height (e.g. \"-d 640x480\")"),
        ("f", "flip x & y axes in plot"),
        ("h|help", "print help message"),
        ("l|log", "any of 'x' or 'y' to log scale"),
        ("m|mode", "'dot'or 'count'. Default 'dot'"),
        (
            "cdf",
            "cumulative distribution function, only for the y value. Not compatible with log & x",
        ),
        ("x", "treat first column as X for all following Y columns"),
    ];
    for (cmd, desc) in commands {
        println!(
            "   \x1b[{}m{:<12}\x1b[0m \x1b[0m{}",
            commands_color, cmd, desc
        )
    }
    std::process::exit(0);
}
