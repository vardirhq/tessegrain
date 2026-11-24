# A couch inspired by the reference chair

This two-seat couch is a separate drawing built with Tessegrain polygons and lines. It borrows the reference chair's isometric direction, wood colors and pixel scale, and adds a muted green fabric palette to make the cushions and backrest distinct.

The first attempt resembled a wooden bench. The revised version has shorter feet, a deeper upholstered seat, two back cushions, and connected armrests. The couch was revised again after checking its pixel contours: all diagonal edges in its current polygon and line geometry use a 2:1 horizontal-to-vertical ratio. This is a geometry check, not proof that every visible contour has perfect pixel clusters. The side-by-side preview is a visual check of style and proportions, not an objective style-match benchmark.

- `couch.json`: editable Tessegrain source, 60×48 pixels.
- `couch.png`: native-size transparent PNG.
- `comparison.png`: reference chair and couch at the same 8× nearest-neighbor scale, aligned along the baseline.

```sh
cargo run -- validate examples/matching-couch/couch.json
cargo run -- render examples/matching-couch/couch.json --output couch.png
```

The chair shown in the comparison is cropped from [Isometric Furniture and Walls](https://opengameart.org/content/isometric-furniture-and-walls) by hawkbirdtree, listed by the creator as CC0. The couch was drawn separately using the chair as a visual reference.
