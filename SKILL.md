---
name: tessegrain
description: Draw and edit experimental pixel art using Tessegrain.
---
# Tessegrain agent guide

Use `cargo run -- render SOURCE --output PREVIEW.png --scale 8` to render a nearest-neighbor preview. Use `cargo run -- import IMAGE.png --output SOURCE.json` for an exact decoded RGBA pixel-grid import and `cargo run -- validate SOURCE.json` to check rendering. Commands and layers draw in order. Grid colors named `transparent` skip pixels; named palette entries write pixels even when alpha is zero. See README.md and examples/chair.json for the format.

Plan the silhouette, palette, perspective and lighting before drawing. Start with large shapes, then use grids and pixels for details. Inspect the rendered result if image viewing is available, revise, and export at scale 1. The tool does not interpret the prompt or judge artistic quality. An imported image does not contain reconstructed parts; preserve the original and use diff tools or a pixel comparison to verify edits.
