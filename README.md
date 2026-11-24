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

Supported commands: `pixel`, `rect` (width and height), `line` (integer endpoints), and `grid` (equal-width character rows and a symbol-to-palette mapping). Operations are clipped to the canvas. See [examples/chair.json](examples/chair.json). The included example is a simple front-facing chair to exercise the format, not a quality benchmark.

## Next experiment

Compare an agent drawing the same isometric chair and matching sofa using this format, a plain drawing script, and relevant references. Record outputs, prompts, model, iterations and human ratings. This tests whether dedicated tools help before investing in an expanded engine.
