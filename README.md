# Tessegrain

Tessegrain is an experiment in making pixel art programmable and editable by AI agents. This first prototype tests a hybrid source format: ordered drawing commands combine shapes and exact pixel grids. The renderer and CLI contain no AI; an agent supplies the creative decisions.

This is an early proof of concept, not a finished pixel editor or evidence that agents can reliably create attractive art. The format may change.

## Try it

Install a current stable Rust toolchain, then:

```sh
cargo run -- render examples/chair.json --output chair.png --scale 8
cargo run -- validate examples/chair.json
cargo run -- import chair.png --output imported.json
cargo run -- render imported.json --output roundtrip.png
cargo test
```

Rendering and imports are local. Building requires downloading dependencies unless they are already cached. `--scale` uses nearest-neighbor scaling; omit it for native resolution. Output is PNG. Import preserves decoded RGBA values, including colors in fully transparent pixels, but not PNG metadata or original layers. It does not recover shapes or semantic parts. The grid currently uses Unicode symbols; the format is experimental.

## Format

Documents have `version: 1`, integer `canvas` dimensions, named hexadecimal RGB or RGBA palette entries, and ordered layers containing ordered commands. The canvas starts transparent. Commands write RGBA values directly (including partially transparent values); there is no alpha blending between layers. Transparent grid symbols skip drawing; use a named transparent palette color to write transparent pixels.

Supported commands: `pixel`, `rect` (width and height), `line` (integer endpoints), `polygon` (3–256 vertices), and `grid` (equal-width character rows and a symbol-to-palette mapping). Operations are clipped to the canvas. Polygons use the even–odd fill rule, testing pixel centers with half-open edge crossings; vertices lie on pixel boundaries. See [examples/chair.json](examples/chair.json). See also [the isometric chair source](examples/isometric-chair.json) and its [8× preview](examples/isometric-chair.png). These examples exercise the format; they are not quality benchmarks.

## Next experiment

Compare an agent drawing the same isometric chair and matching sofa using this format, a plain drawing script, and relevant references. Record outputs, prompts, model, iterations and human ratings. This tests whether dedicated tools help before investing in an expanded engine.

## Recreation experiment

See the [isometric chair recreation experiment](benchmarks/isometric-chair/README.md) for a CC0 reference, an editable redraw, a side-by-side preview, and reproducible pixel comparisons.

## Inspect rendered pixels

`inspect` renders an artwork JSON, or reads a PNG, and prints a JSON report:

```sh
cargo run -- inspect examples/matching-couch/couch.json
cargo run -- inspect examples/matching-couch/couch.png --region 0,0,60,48
cargo run -- inspect examples/matching-couch/couch.png --color '#8cb2a5'
```

The optional region is `x,y,width,height`, must be entirely inside the canvas, and output coordinates remain canvas-relative. `--color '#RRGGBB'` selects that exact RGB with any nonzero alpha; without it, all pixels with alpha greater than zero count as visible. Counts and colors are for the selected region and color mask. A partially transparent pixel counts as visible, and isolated means no other selected pixel in its eight neighboring positions **inside the selected region**. Thus crop boundaries can create isolation.

The `top` and `bottom` reports scan each column for its first and last selected pixel in the region. They group consecutive columns at the same y into horizontal runs, breaking chains at empty columns, vertical jumps larger than one pixel, or direction changes. An observation flags an interior run when its immediate neighbors are the same length but it differs. A run's inclusive `x_start` and `x_end` and `y` locate it in the source canvas; `chain` distinguishes discontinuous paths. `selection.rgb` records the optional mask.

These are **observations, not errors or aesthetic scores**. Endpoint runs are intentionally not flagged, deliberate changes can look irregular, and clipping a region can change results. Only upper/lower outermost contours of the selected mask are examined: this does not trace holes, multiple intervals per column, vertical run patterns, or all internal color boundaries. A color mask can expose some internal shapes, but visually inspect the output. Selecting a tight region around an edge can make the report easier to interpret.

For example, the couch's full silhouette reports a possible bottom-edge interruption at `(49..50, 24)` in the current image. [This preview](examples/inspection/couch-observation.png) marks the reported run in red. It is a location to inspect, not a claim that this corner should be changed.
