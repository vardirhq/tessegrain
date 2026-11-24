use image::{Rgba, RgbaImage};
use tessegrain::{import_image, render, Document};

#[test]
fn shapes_and_grids_compose_in_order() {
    let doc: Document = serde_json::from_str(
        r##"{
      "version":1,"canvas":{"width":3,"height":2},
      "palette":{"red":"#ff0000","blue":"#0000ff"},
      "layers":[{"name":"art","commands":[
        {"op":"rect","x":0,"y":0,"w":3,"h":2,"color":"red"},
        {"op":"grid","x":1,"y":0,"colors":{".":"transparent","X":"blue"},"rows":["X.",".X"]}
      ]}]}"##,
    )
    .unwrap();
    let im = render(&doc).unwrap();
    assert_eq!(im.get_pixel(1, 0), &Rgba([0, 0, 255, 255]));
    assert_eq!(im.get_pixel(2, 0), &Rgba([255, 0, 0, 255]));
    assert_eq!(im.get_pixel(2, 1), &Rgba([0, 0, 255, 255]));
}

#[test]
fn import_preserves_all_rgba_values() {
    let mut im = RgbaImage::new(2, 1);
    im.put_pixel(0, 0, Rgba([20, 30, 40, 0]));
    im.put_pixel(1, 0, Rgba([12, 34, 56, 123]));
    assert_eq!(render(&import_image(&im).unwrap()).unwrap(), im);
}

#[test]
fn undefined_grid_symbols_are_errors() {
    let doc: Document = serde_json::from_str(r#"{"version":1,"canvas":{"width":1,"height":1},"palette":{},"layers":[{"name":"x","commands":[{"op":"grid","x":0,"y":0,"colors":{},"rows":["X"]}]}]}"#).unwrap();
    assert!(render(&doc).unwrap_err().contains("grid"));
}

#[test]
fn polygon_fills_pixel_centers_with_even_odd_rule_and_clips() {
    let doc: Document = serde_json::from_str(r##"{"version":1,"canvas":{"width":4,"height":4},"palette":{"ink":"#ff0000"},"layers":[{"name":"x","commands":[{"op":"polygon","points":[[-1,0],[3,0],[3,3],[-1,3]],"color":"ink"}]}]}"##).unwrap();
    let im = render(&doc).unwrap();
    assert_eq!(im.get_pixel(0, 0), &Rgba([255, 0, 0, 255]));
    assert_eq!(im.get_pixel(2, 2), &Rgba([255, 0, 0, 255]));
    assert_eq!(im.get_pixel(3, 2), &Rgba([0, 0, 0, 0]));
    assert_eq!(im.get_pixel(1, 3), &Rgba([0, 0, 0, 0]));
}

#[test]
fn polygon_rejects_too_few_vertices() {
    let doc: Document = serde_json::from_str(r##"{"version":1,"canvas":{"width":2,"height":2},"palette":{"ink":"#ff0000"},"layers":[{"name":"x","commands":[{"op":"polygon","points":[[0,0],[1,1]],"color":"ink"}]}]}"##).unwrap();
    assert!(render(&doc).unwrap_err().contains("polygon"));
}

#[test]
fn concave_polygon_leaves_its_notch_empty() {
    let doc: Document = serde_json::from_str(r##"{"version":1,"canvas":{"width":4,"height":4},"palette":{"ink":"#ff0000"},"layers":[{"name":"x","commands":[{"op":"polygon","points":[[0,0],[4,0],[4,1],[1,1],[1,4],[0,4]],"color":"ink"}]}]}"##).unwrap();
    let im = render(&doc).unwrap();
    assert_eq!(im.get_pixel(3, 0), &Rgba([255, 0, 0, 255]));
    assert_eq!(im.get_pixel(0, 3), &Rgba([255, 0, 0, 255]));
    assert_eq!(im.get_pixel(2, 2), &Rgba([0, 0, 0, 0]));
}
