# Isometric box furniture

The chair and couch use the same two face-color materials and depth-tested box primitives. The couch contains four legs, a seat frame, a back, two arms, and two seat cushions. The chair uses the same component arrangement with a narrower seat. Render with:

```sh
cargo run -- render examples/isometric-boxes/chair.json --output chair.png
cargo run -- render examples/isometric-boxes/couch.json --output couch.png
```

`comparison.png` shows these at the same pixel scaling as the earlier polygon couch. The box approach makes projection, shared material colors, and overlaps easier to maintain; the result is visibly more blocky and doesn't automatically create the earlier couch's outlines or details. Coordinates remain explicit, and these examples do not implement reusable or parameterized parts. This is an exploratory example, not a timed, controlled benchmark.

## Detail pass

[`couch-detailed.json`](couch-detailed.json) preserves the original scene and materials, then adds a separate 2D layer for back cushions, a seat seam, and wood highlights. [Compare the original and detailed renders](detail-comparison.png). The silhouette is unchanged; the additions stay within visible pixels of the original render.

```sh
cargo run -- render examples/isometric-boxes/couch-detailed.json --output couch-detailed.png
```

The detail layer uses absolute screen coordinates and draws over the projected scene. It is useful for manual finishing, but it is not attached to individual boxes: changing the origin, perspective, or dimensions may misalign details. The two JSON documents duplicate the scene, so edits to the base are not automatically propagated. The example demonstrates composition, not a reusable-part or surface-anchoring feature.
