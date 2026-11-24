# Isometric chair recreation experiment

The reference is a 32×32 crop at (32, 192) from [Isometric Furniture and Walls](https://opengameart.org/content/isometric-furniture-and-walls), by **hawkbirdtree**, listed as **CC0** by the creator on OpenGameArt. `reference.png` preserves the crop's RGBA values. The larger sheet also contains other matching furniture.

The goal was to redraw the chair from the reference using Tessegrain's editable polygon and line primitives. `recreation.json` was constructed by an AI agent in a single interactive session after visual inspection and examination of the pixel grid; its palette matches the reference. It was not produced by importing the reference into Tessegrain. This is one exploratory attempt, not a controlled benchmark or evidence of generalization. The source and result are included for review and future revisions.

## Reproduce

From the repository root:

```sh
cargo run -- render benchmarks/isometric-chair/recreation.json --output benchmarks/isometric-chair/recreation.png
python -m pip install Pillow
python benchmarks/isometric-chair/compare.py
```

The comparison script reports silhouette intersection-over-union (IoU) and exact RGBA matches, both over the union of nontransparent pixels. Fully transparent RGB is intentionally ignored, because it is invisible and the imported reference uses white in its transparent pixels. For this opaque reference, IoU measures overlapping occupied pixels; it is not an artistic quality score. The pixel match uses the visible union as denominator to avoid an inflated result from the empty background. The comparison image uses 8× nearest-neighbor previews over white.

## Result

After inspecting the first render and adjusting its shapes, this reconstruction achieves 243/271 (89.67%) silhouette IoU and 207/271 (76.38%) exact RGBA matches over the visible union. It captures the overall form but differs in the backrest, seat edges, and shading. The geometry and pixel colors were examined during the attempt, so this score does not measure performance without pixel-level feedback. It does not test a matching new object or generalization to other sprites.
