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
