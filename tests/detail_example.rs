use tessegrain::{render, Document};

#[test]
fn detailed_couch_keeps_scene_and_silhouette_while_adding_visible_detail() {
    let base_source = include_str!("../examples/isometric-boxes/couch.json");
    let detail_source = include_str!("../examples/isometric-boxes/couch-detailed.json");
    let base_json: serde_json::Value = serde_json::from_str(base_source).unwrap();
    let detail_json: serde_json::Value = serde_json::from_str(detail_source).unwrap();
    assert_eq!(base_json["layers"][0], detail_json["layers"][0]);
    assert_eq!(base_json["materials"], detail_json["materials"]);
    let base: Document = serde_json::from_str(base_source).unwrap();
    let detail: Document = serde_json::from_str(detail_source).unwrap();
    let first = render(&base).unwrap();
    let second = render(&detail).unwrap();
    assert_eq!(first.dimensions(), second.dimensions());
    assert!(first.pixels().zip(second.pixels()).any(|(a, b)| a != b));
    assert!(first
        .pixels()
        .zip(second.pixels())
        .all(|(a, b)| a[3] == b[3]));
}
