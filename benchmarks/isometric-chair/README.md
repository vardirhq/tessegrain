# Isometric chair recreation experiment

The reference is a 32×32 crop at (32, 192) from [Isometric Furniture and Walls](https://opengameart.org/content/isometric-furniture-and-walls), by **hawkbirdtree**, listed as **CC0** on OpenGameArt. `reference.png` preserves the crop's RGBA values. The larger sheet also contains matching furniture.

## Two stages

- `baseline.json` and `baseline.png` preserve the initial agent-drawn recreation. The agent used polygons and lines, examined the reference's pixel grid, and adjusted the geometry. It was not created using the PNG import command.
- `recreation.json` keeps those shapes and adds a grid of 64 pixel corrections, **calculated directly by comparing the baseline with the reference**. It writes the reference color where visible pixels differed and erases pixels outside the reference silhouette. `recreation.png` is the resulting render.

The corrected version is a pixel trace assisted by access to the exact reference. Its exact match demonstrates that shapes and direct pixel edits can coexist and remain editable; it does not demonstrate independent artistic judgment or an ability to generate matching new objects. An imported reference grid could also reproduce the reference exactly, so 100% is not a meaningful standalone quality benchmark. Keeping the baseline provides an honest measure before automatic corrections.

## Reproduce

From the repository root:

```sh
cargo run -- render benchmarks/isometric-chair/baseline.json --output benchmarks/isometric-chair/baseline.png
cargo run -- render benchmarks/isometric-chair/recreation.json --output benchmarks/isometric-chair/recreation.png
python -m pip install Pillow
python benchmarks/isometric-chair/compare.py
```

The comparison script reports silhouette intersection-over-union (IoU) and exact RGBA matches, both over the union of nontransparent pixels. Fully transparent RGB is intentionally ignored, because it is invisible and the reference uses white in its transparent pixels. For this opaque reference, IoU measures overlapping occupied pixels; it is not an artistic quality score. The pixel match uses the visible union as denominator to avoid inflating the result with the empty background. The comparison image uses 8× nearest-neighbor previews over white.

## Results

The baseline has 243/271 (89.67%) silhouette IoU and 207/271 (76.38%) exact RGBA matches. The corrected render reaches 249/249 (100%) on both measures. The image is visually identical at native resolution, but fully transparent pixels can retain different RGB values, and PNG metadata is not compared.
