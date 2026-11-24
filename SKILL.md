---
name: tessegrain
description: Draw and edit experimental pixel art using Tessegrain.
---
# Tessegrain agent guide

Use `cargo run -- render SOURCE --output PREVIEW.png --scale 8` to render a nearest-neighbor preview. Use `cargo run -- import IMAGE.png --output SOURCE.json` for an exact decoded RGBA pixel-grid import and `cargo run -- validate SOURCE.json` to check rendering. Commands and layers draw in order. Grid colors named `transparent` skip pixels; named palette entries write pixels even when alpha is zero. Use polygons for angled planes; their vertices are on pixel boundaries and their fill tests pixel centers. See README.md, examples/chair.json, and examples/isometric-chair.json for the format.

Plan the silhouette, palette, perspective and lighting before drawing. Start with large shapes, then use grids and pixels for details. Inspect the rendered result if image viewing is available, revise, and export at scale 1. The tool does not interpret the prompt or judge artistic quality. An imported image does not contain reconstructed parts; preserve the original and use diff tools or a pixel comparison to verify edits.

Use `cargo run -- inspect SOURCE` for JSON observations of rendered pixels. Restrict to `--region x,y,width,height` or `--color '#RRGGBB'` to investigate a particular region or color. The edge report only follows the outermost top and bottom visible pixels in each column; it does not analyze every contour. Coordinates are canvas-relative. A flagged run is a possible interruption, not an instruction to change art. Compare it with a native-size and nearest-neighbor preview and use artistic judgment.

For block-based isometric furniture, use `iso_scene` with material face colors. Its boxes share a depth buffer and a 2:1 projection; consult the README and `examples/isometric-boxes/` for coordinates. Start with one scene per object and position components in grid space. The optional outline covers only the combined silhouette. Check the actual render: coherent coordinates alone do not guarantee good silhouettes, appealing shapes, or clean pixel clusters. Add ordinary pixel/grid operations afterwards when needed.

For rounded or stepped isometric parts, `iso_scene` entries can set `shape` to `cylinder`, `stairs_x`, or `stairs_y`; see README.md and `examples/isometric-shapes/`. Cylinders are grid-stepped, not smooth; stairs require a `steps` count, and `reverse: true` reverses their rise so treads can face the viewer. All components in one scene use the same depth test and materials. Inspect previews for faceting, proportions, and unintended overlaps.
