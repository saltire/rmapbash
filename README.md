# rmapbash
## Minecraft map renderer

Reads a saved Minecraft world from disk and outputs a rendered .PNG image.

![Isometric day mode](./samples/iso-day.png?raw=true)

![Isometric night mode](./samples/iso-night.png?raw=true)

![Orthographic day mode](./samples/ortho-day.png?raw=true)

![Orthographic night mode](./samples/ortho-night.png?raw=true)

### Features so far

- Orthographic (top-down) or isometric (oblique) viewing angle.
- Day or night lighting mode.
- Nether and End supported; just point to the `DIM-1` or `DIM1` subdir of the save dir.
- Render part of a world by passing coordinates at two corners of a bounding box;
  e.g. `-b 10 20 200 400` to render only the area between (10, 20) and (200, 400).
- Render a vertical slice by passing min/max Y values; e.g. `-y 20 100`.

```
USAGE:
    rmapbash [FLAGS] [OPTIONS] <PATH>

FLAGS:
    -h, --help         Prints help information
    -i, --isometric    Isometric view
    -n, --night        Night lighting
    -V, --version      Prints version information

OPTIONS:
    -b, --blocks <N> <W> <S> <E>    Horizontal block limits
    -y, --yblocks <MIN> <MAX>       Vertical block limits

ARGS:
    <PATH>    Path to either a save directory or a .dat file
```

### About

This is my first project in Rust; I've been using it to learn the language.

It's a reimplementation of my first C project, [cmapbash](https://github.com/saltire/cmapbash),
which is what I used to learn *that* language.
