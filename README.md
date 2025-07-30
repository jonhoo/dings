# dings: quick command-line data visualization

dings reads a stream of points from a file / stdin and plots them.

dings is a Rust port of
[silentbicycle/guff](https://github.com/silentbicycle/guff).
Much of the credit goes to the original author, Scott Vokes.

dings is distributed under the [ISC
License](https://opensource.org/licenses/isc-license.txt) as guff was.

The initial implementation was done [as a
livestream](https://youtu.be/bbWcGAOsbIE).

## TODOs

**Argument help text**: Fill in the text for `--help` (currently
`unimplemented!`). Ideally we would also generate a man page.

**Improved README**: Usage examples, differences from guff, how to
install.

**Colored output**: Terminal escape codes to color datasets and axes.
Ideally using <https://colorbrewer2.org/>.

**Remaining features from guff**: `-f` to flip X and Y, `-log c` to get
logarithmic count ("the trick" won't work any more), `-S` to disable
stream mode, `file` argument to read from file, and support for blank
lines to reset. Notably, probably not SVG.

**Tests**: The original had extensive tests that we should bring over.
Fuzzing, probably with `quickcheck`, but maybe AFL for input fuzzing as
well.

**Additional transformations**: Support for
[PDF](https://en.wikipedia.org/wiki/Probability_density_function).

**Statically linked binary release**: A tool like this it's super handy
to be able to just fetch and run. Probably best managed with
<https://github.com/axodotdev/cargo-dist>.
