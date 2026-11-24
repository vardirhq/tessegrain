# Isometric shape examples

The barrel and round table combine upright grid-stepped cylinders with wood and metal materials. The stairs use `stairs_x` with five discrete steps. All use the existing scene projection, depth buffer, and silhouette outline. The examples are constructed from shapes in grid space; they do not use hand-placed 2D details.

```sh
cargo run -- render examples/isometric-shapes/barrel.json --output barrel.png
cargo run -- render examples/isometric-shapes/round-table.json --output round-table.png
cargo run -- render examples/isometric-shapes/stairs.json --output stairs.png
```

The barrel is assembled from stacked wood and metal segments so its bands remain visible: putting a thinner colored cylinder inside a solid one would hide it. `showcase.png` displays all three at 6× using nearest-neighbor scaling. Circular surfaces are approximated using grid cells, resulting in visible facets and steps. The staircase is not a smooth wedge.
