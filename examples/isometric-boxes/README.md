# Isometric box furniture

The chair and couch use the same two face-color materials and depth-tested box primitives. The couch contains four legs, a seat frame, a back, two arms, and two seat cushions. The chair uses the same component arrangement with a narrower seat. Render with:

```sh
cargo run -- render examples/isometric-boxes/chair.json --output chair.png
cargo run -- render examples/isometric-boxes/couch.json --output couch.png
```

`comparison.png` shows these at the same pixel scaling as the earlier polygon couch. The box approach makes projection, shared material colors, and overlaps easier to maintain; the result is visibly more blocky and doesn't automatically create the earlier couch's outlines or details. Coordinates remain explicit, and these examples do not implement reusable or parameterized parts. This is an exploratory example, not a timed, controlled benchmark.
