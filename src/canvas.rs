use std::fmt;

#[derive(Debug, Default, Copy, Clone)]
pub(crate) enum Mode {
    #[default]
    Dot,
    Count,
}

#[derive(Debug)]
pub(crate) struct Canvas {
    stride: usize,
    cells: Box<[u8]>,
    pub(crate) mode: Mode,
}

impl fmt::Display for Canvas {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in self.rows() {
            write!(
                f,
                "{}",
                std::str::from_utf8(row).expect("row is invalid utf-8")
            )?;
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Canvas {
    pub(crate) fn new(rows: usize, columns: usize, mode: Mode) -> Self {
        Self {
            stride: columns,
            cells: vec![b' '; rows * columns].into_boxed_slice(),
            mode,
        }
    }

    pub(crate) fn rows(&self) -> impl Iterator<Item = &[u8]> {
        self.cells.chunks_exact(self.stride)
    }

    pub(crate) fn cell(&mut self, row: usize, column: usize) -> Option<&mut u8> {
        self.cells.get_mut(row * self.stride + column)
    }
}
