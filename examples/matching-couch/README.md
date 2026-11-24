# A couch to match the reference chair

This wooden settee is a new drawing built from polygons and lines with the same four-color palette and a comparable isometric perspective to the [reference chair](../../benchmarks/isometric-chair/README.md). It is an exploratory attempt to make a new object in the same style, not a recreation or a claim of an objectively verified style match.

- `couch.json`: editable Tessegrain source, 48×40 pixels.
- `couch.png`: native-size transparent PNG.
- `comparison.png`: the reference chair and couch at the same 8× nearest-neighbor scale, aligned along the baseline.

```sh
cargo run -- validate examples/matching-couch/couch.json
cargo run -- render examples/matching-couch/couch.json --output couch.png
```

The chair shown in the comparison is cropped from [Isometric Furniture and Walls](https://opengameart.org/content/isometric-furniture-and-walls) by hawkbirdtree, listed by the creator as CC0. The couch was drawn separately using the chair as a visual reference.
