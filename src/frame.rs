use crate::{Canvas, Data};

pub const PAD: usize = 2;

pub(crate) struct Frame {
    width: usize,
    height: usize,

    min_x: f64,
    max_x: f64,
    range_x: f64,

    min_y: f64,
    max_y: f64,
    range_y: f64,
}

impl Frame {
    pub(crate) fn new_over(width: usize, height: usize, data: &Data) -> Self {
        let min_x = data
            .xs
            .iter()
            .filter(|v| v.is_finite())
            .copied()
            .min_by(f64::total_cmp);
        let max_x = data
            .xs
            .iter()
            .filter(|v| v.is_finite())
            .copied()
            .max_by(f64::total_cmp);
        let min_y = data
            .ys
            .iter()
            .flatten()
            .filter(|v| v.is_finite())
            .copied()
            .min_by(f64::total_cmp);
        let max_y = data
            .ys
            .iter()
            .flatten()
            .filter(|v| v.is_finite())
            .copied()
            .max_by(f64::total_cmp);

        let (Some(mut min_x), Some(max_x)) = (min_x, max_x) else {
            todo!();
        };
        let (Some(mut min_y), Some(max_y)) = (min_y, max_y) else {
            todo!();
        };

        /* Override bounds that would lead to a range of zero, to avoid a
         * crash when plotting. (Found by afl.) */
        let mut max_x = if min_x == max_x { min_x + 1. } else { max_x };
        let mut max_y = if min_y == max_y { min_y + 1. } else { max_y };

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

        Self {
            width,
            height,
            min_x,
            max_x,
            range_x,
            min_y,
            max_y,
            range_y,
        }
    }

    pub(crate) fn x_bounds(&self) -> (f64, f64) {
        (self.min_x, self.max_x)
    }

    pub(crate) fn y_bounds(&self) -> (f64, f64) {
        (self.min_y, self.max_y)
    }

    pub(crate) fn range_xy(&self) -> (f64, f64) {
        (self.range_x, self.range_y)
    }

    pub(crate) fn x_to_column(&self, x: f64) -> usize {
        let plot_width = (self.width - PAD) as f64;
        let x_as_fraction_of_axis = (x - self.min_x) / self.range_x;
        (plot_width * x_as_fraction_of_axis).round() as usize
    }

    pub(crate) fn y_to_row(&self, y: f64) -> usize {
        let plot_height = (self.height - PAD) as f64;
        let y_as_fraction_of_axis = (y - self.min_y) / self.range_y;
        let y_cell_from_top = (plot_height * y_as_fraction_of_axis).round() as usize;
        // flip y; 0 at bottom of plot
        self.height - y_cell_from_top - 1
    }

    pub(crate) fn point_to_cell(&self, (x, y): (f64, f64)) -> (usize, usize) {
        (self.y_to_row(y), self.x_to_column(x))
    }

    pub(crate) fn draw_into(&self, canvas: &mut Canvas) {
        // figure out where to draw the axes in the frame
        let y0_is_visible = self.min_y <= 0. && self.max_y >= 0.;
        let x0_is_visible = self.min_x <= 0. && self.max_x >= 0.;

        let mut draw_vertical_at_x = 0.;
        let mut draw_horizontal_at_y = 0.;
        if !x0_is_visible {
            if self.min_x > 0. {
                draw_vertical_at_x = self.min_x;
            } else {
                draw_vertical_at_x = self.max_x;
            }
        }
        if !y0_is_visible {
            if self.min_y > 0. {
                draw_horizontal_at_y = self.min_y;
            } else {
                draw_horizontal_at_y = self.max_y;
            }
        }

        let draw_horizontal_at_row = self.point_to_cell((0., draw_horizontal_at_y)).0;
        let draw_vertical_at_column = self.point_to_cell((draw_vertical_at_x, 0.)).1;

        // draw in the axes
        // draw the vertical (Y) axis (so where X = 0)
        for row in 0..self.height {
            #[allow(clippy::collapsible_else_if)]
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
        for column in 0..self.width {
            #[allow(clippy::collapsible_else_if)]
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
    }
}
