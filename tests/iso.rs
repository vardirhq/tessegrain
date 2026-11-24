use image::Rgba;
use tessegrain::{render, Document};

fn doc(boxes: &str, outline: bool) -> Document {
    serde_json::from_str(&format!(r##"{{"version":1,"canvas":{{"width":40,"height":40}},"palette":{{"top":"#ff0000","left":"#00ff00","right":"#0000ff","ink":"#000000"}},"materials":{{"wood":{{"top":"top","left":"left","right":"right","outline":"ink"}}}},"layers":[{{"name":"scene","commands":[{{"op":"iso_scene","origin":[20,20],"outline":{outline},"boxes":{boxes}}}]}}]}}"##)).unwrap()
}
#[test]
fn box_has_projected_faces_with_material_colors() {
    let d = doc(
        r#"[{"x":0,"y":0,"z":0,"w":4,"d":4,"h":4,"material":"wood"}]"#,
        false,
    );
    let im = render(&d).unwrap();
    assert_eq!(*im.get_pixel(20, 15), Rgba([255, 0, 0, 255]));
    assert_eq!(*im.get_pixel(24, 21), Rgba([0, 255, 0, 255]));
    assert_eq!(*im.get_pixel(16, 21), Rgba([0, 0, 255, 255]));
    assert_eq!(*im.get_pixel(0, 0), Rgba([0, 0, 0, 0]));
}
#[test]
fn overlapping_boxes_do_not_depend_on_input_order() {
    let a = r#"{"x":0,"y":0,"z":0,"w":4,"d":4,"h":3,"material":"wood"}"#;
    let b = r#"{"x":2,"y":2,"z":2,"w":3,"d":3,"h":5,"material":"wood"}"#;
    assert_eq!(
        render(&doc(&format!("[{a},{b}]"), false)).unwrap(),
        render(&doc(&format!("[{b},{a}]"), false)).unwrap()
    );
}
#[test]
fn outlines_are_optional_and_do_not_erase_the_box() {
    let b = r#"[{"x":0,"y":0,"z":0,"w":4,"d":4,"h":4,"material":"wood"}]"#;
    let off = render(&doc(b, false)).unwrap();
    let on = render(&doc(b, true)).unwrap();
    assert!(on.pixels().any(|p| *p == Rgba([0, 0, 0, 255])));
    assert!(off.pixels().all(|p| *p != Rgba([0, 0, 0, 255])));
    assert!(on.pixels().any(|p| *p == Rgba([255, 0, 0, 255])));
}
#[test]
fn invalid_box_and_unknown_material_are_errors() {
    let zero = doc(
        r#"[{"x":0,"y":0,"z":0,"w":0,"d":4,"h":4,"material":"wood"}]"#,
        false,
    );
    assert!(render(&zero).unwrap_err().contains("dimensions"));
    let missing = doc(
        r#"[{"x":0,"y":0,"z":0,"w":1,"d":1,"h":1,"material":"missing"}]"#,
        false,
    );
    assert!(render(&missing).unwrap_err().contains("material"));
}
#[test]
fn old_documents_remain_compatible() {
    let d:Document=serde_json::from_str(r##"{"version":1,"canvas":{"width":1,"height":1},"palette":{"red":"#ff0000"},"layers":[{"name":"x","commands":[{"op":"pixel","x":0,"y":0,"color":"red"}]}]}"##).unwrap();
    assert_eq!(*render(&d).unwrap().get_pixel(0, 0), Rgba([255, 0, 0, 255]));
}

#[test]
fn different_materials_are_depth_sorted_independent_of_order() {
    let a = r#"{"x":0,"y":0,"z":0,"w":5,"d":5,"h":3,"material":"wood"}"#;
    let b = r#"{"x":2,"y":2,"z":2,"w":3,"d":3,"h":5,"material":"other"}"#;
    fn with_other(mut d: Document) -> Document {
        d.materials.insert(
            "other".into(),
            tessegrain::iso::Material {
                top: "right".into(),
                left: "top".into(),
                right: "left".into(),
                outline: "ink".into(),
            },
        );
        d
    }
    let forward = render(&with_other(doc(&format!("[{a},{b}]"), false))).unwrap();
    let backward = render(&with_other(doc(&format!("[{b},{a}]"), false))).unwrap();
    assert_eq!(forward, backward);
}

#[test]
fn clipped_box_renders_and_scene_preserves_existing_background() {
    let mut d = doc(
        r#"[{"x":0,"y":0,"z":0,"w":4,"d":4,"h":4,"material":"wood"}]"#,
        false,
    );
    if let tessegrain::Command::IsoScene { ref mut origin, .. } = d.layers[0].commands[0] {
        *origin = [1, 5];
    } else {
        panic!("expected scene")
    }
    d.layers[0].commands.insert(
        0,
        tessegrain::Command::Rect {
            x: 0,
            y: 0,
            w: 40,
            h: 40,
            color: "left".into(),
        },
    );
    let im = render(&d).unwrap();
    assert_eq!(*im.get_pixel(39, 39), Rgba([0, 255, 0, 255]));
    assert!(im.pixels().any(|p| *p == Rgba([255, 0, 0, 255])));
}

#[test]
fn transparent_scene_material_is_rejected() {
    let mut d = doc(
        r#"[{"x":0,"y":0,"z":0,"w":4,"d":4,"h":4,"material":"wood"}]"#,
        false,
    );
    d.palette.insert("top".into(), "#ff000080".into());
    assert!(render(&d).unwrap_err().contains("opaque"));
}

#[test]
fn touching_boxes_share_a_silhouette_without_internal_outlines() {
    let whole = doc(
        r#"[{"x":0,"y":0,"z":0,"w":4,"d":4,"h":4,"material":"wood"}]"#,
        true,
    );
    let split = doc(
        r#"[{"x":0,"y":0,"z":0,"w":2,"d":4,"h":4,"material":"wood"},{"x":2,"y":0,"z":0,"w":2,"d":4,"h":4,"material":"wood"}]"#,
        true,
    );
    assert_eq!(render(&whole).unwrap(), render(&split).unwrap());
}

#[test]
fn cylinder_has_rounder_footprint_than_its_bounding_box() {
    let cylinder = doc(
        r#"[{"shape":"cylinder","x":0,"y":0,"z":0,"w":8,"d":8,"h":3,"material":"wood"}]"#,
        false,
    );
    let bounding = doc(
        r#"[{"x":0,"y":0,"z":0,"w":8,"d":8,"h":3,"material":"wood"}]"#,
        false,
    );
    let a = render(&cylinder).unwrap();
    let b = render(&bounding).unwrap();
    let visible = |image: &image::RgbaImage| image.pixels().filter(|p| p[3] > 0).count();
    assert!(visible(&a) > 0 && visible(&a) < visible(&b));
}

#[test]
fn stairs_produce_several_height_levels() {
    let stair = doc(
        r#"[{"shape":"stairs_x","steps":4,"x":0,"y":0,"z":0,"w":8,"d":4,"h":8,"material":"wood"}]"#,
        false,
    );
    let block = doc(
        r#"[{"x":0,"y":0,"z":0,"w":8,"d":4,"h":8,"material":"wood"}]"#,
        false,
    );
    assert_ne!(render(&stair).unwrap(), render(&block).unwrap());
}

#[test]
fn invalid_step_count_is_rejected() {
    let d = doc(
        r#"[{"shape":"stairs_x","steps":9,"x":0,"y":0,"z":0,"w":4,"d":4,"h":4,"material":"wood"}]"#,
        false,
    );
    assert!(render(&d).unwrap_err().contains("steps"));
}

#[test]
fn unknown_shape_is_rejected_at_parse_time() {
    let mut d: serde_json::Value = serde_json::to_value(doc(
        r#"[{"x":0,"y":0,"z":0,"w":4,"d":4,"h":4,"material":"wood"}]"#,
        false,
    ))
    .unwrap();
    d["layers"][0]["commands"][0]["boxes"][0]["shape"] = "typo".into();
    assert!(serde_json::from_value::<Document>(d).is_err());
}

#[test]
fn expanded_shapes_accept_maximum_valid_input_coordinates() {
    let d = doc(
        r#"[{"shape":"cylinder","x":4096,"y":0,"z":0,"w":8,"d":8,"h":3,"material":"wood"}]"#,
        false,
    );
    assert!(render(&d).is_ok());
}
#[test]
fn expanded_shapes_share_depth_order_with_boxes() {
    let a = r#"{"shape":"cylinder","x":0,"y":0,"z":0,"w":6,"d":6,"h":3,"material":"wood"}"#;
    let b =
        r#"{"shape":"stairs_y","steps":3,"x":1,"y":1,"z":1,"w":3,"d":6,"h":6,"material":"wood"}"#;
    assert_eq!(
        render(&doc(&format!("[{a},{b}]"), false)).unwrap(),
        render(&doc(&format!("[{b},{a}]"), false)).unwrap()
    );
}

#[test]
fn stairs_y_and_x_project_differently() {
    let x = doc(
        r#"[{"shape":"stairs_x","steps":3,"x":0,"y":0,"z":0,"w":6,"d":6,"h":6,"material":"wood"}]"#,
        false,
    );
    let y = doc(
        r#"[{"shape":"stairs_y","steps":3,"x":0,"y":0,"z":0,"w":6,"d":6,"h":6,"material":"wood"}]"#,
        false,
    );
    assert_ne!(
        render(&x).unwrap().into_raw(),
        render(&y).unwrap().into_raw()
    );
}
#[test]
fn shape_parameters_are_validated() {
    for (input, expected) in [
        (
            r#"[{"shape":"cylinder","x":0,"y":0,"z":0,"w":65,"d":4,"h":2,"material":"wood"}]"#,
            "cylinder",
        ),
        (
            r#"[{"shape":"stairs_y","x":0,"y":0,"z":0,"w":2,"d":4,"h":4,"material":"wood"}]"#,
            "steps",
        ),
        (
            r#"[{"shape":"box","steps":1,"x":0,"y":0,"z":0,"w":2,"d":4,"h":4,"material":"wood"}]"#,
            "steps",
        ),
        (
            r#"[{"shape":"stairs_y","steps":0,"x":0,"y":0,"z":0,"w":2,"d":4,"h":4,"material":"wood"}]"#,
            "steps",
        ),
    ] {
        assert!(render(&doc(input, false)).unwrap_err().contains(expected));
    }
}
#[test]
fn scene_expansion_limit_is_enforced_before_rendering() {
    let item = r#"{"shape":"cylinder","x":0,"y":0,"z":0,"w":64,"d":64,"h":1,"material":"wood"}"#;
    let d = doc(&format!("[{}]", vec![item; 65].join(",")), false);
    assert!(render(&d).unwrap_err().contains("4096"));
}
#[test]
fn explicit_shape_fields_survive_serialization() {
    let d = doc(
        r#"[{"shape":"stairs_y","steps":3,"x":0,"y":0,"z":0,"w":4,"d":6,"h":6,"material":"wood"}]"#,
        false,
    );
    let copy: Document = serde_json::from_str(&serde_json::to_string(&d).unwrap()).unwrap();
    assert_eq!(
        render(&d).unwrap().into_raw(),
        render(&copy).unwrap().into_raw()
    );
}
