use image::{Rgba, RgbaImage};
use tessegrain::inspect::{inspect, Region};

fn staircase(widths: &[u32]) -> RgbaImage {
    let mut image = RgbaImage::new(widths.iter().sum(), widths.len() as u32);
    let mut x = 0;
    for (y, &width) in widths.iter().enumerate() {
        for _ in 0..width {
            image.put_pixel(x, y as u32, Rgba([1, 2, 3, 255]));
            x += 1;
        }
    }
    image
}

#[test]
fn uniform_staircase_reports_runs_without_irregularities() {
    let image = staircase(&[2, 2, 2, 2, 2]);
    let result = inspect(&image, None).unwrap();
    let top = &result.edges[0];
    assert_eq!(
        top.runs.iter().map(|r| r.length).collect::<Vec<_>>(),
        [2, 2, 2, 2, 2]
    );
    assert!(top.irregularities.is_empty());
}

#[test]
fn interior_outlier_is_reported_at_rendered_location() {
    let image = staircase(&[2, 2, 1, 2, 2]);
    let result = inspect(&image, None).unwrap();
    let issues = &result.edges[0].irregularities;
    assert_eq!(issues.len(), 1);
    assert_eq!(
        (issues[0].x_start, issues[0].y, issues[0].length),
        (4, 2, 1)
    );
}

#[test]
fn empty_columns_break_contours_and_exclude_false_outliers() {
    let mut image = staircase(&[2, 2, 2]);
    for y in 0..image.height() {
        image.put_pixel(2, y, Rgba([0, 0, 0, 0]));
    }
    let result = inspect(&image, None).unwrap();
    assert!(result.edges[0].irregularities.is_empty());
}

#[test]
fn transparent_pixels_are_ignored_and_region_coordinates_are_global() {
    let mut image = staircase(&[2, 2, 1, 2, 2]);
    image.put_pixel(4, 2, Rgba([255, 0, 0, 0]));
    let result = inspect(
        &image,
        Some(Region {
            x: 5,
            y: 0,
            width: 4,
            height: 5,
        }),
    )
    .unwrap();
    assert_eq!(result.visible_pixels, 4);
    assert_eq!(result.edges[0].runs[0].x_start, 5);
    assert!(inspect(
        &image,
        Some(Region {
            x: 8,
            y: 0,
            width: 2,
            height: 1
        })
    )
    .is_err());
}

#[test]
fn cli_returns_machine_readable_json() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/chair.json");
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_tessegrain"))
        .args(["inspect", path, "--region", "0,0,16,16"])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(json["canvas"]["width"], 16);
    assert_eq!(json["edges"].as_array().unwrap().len(), 2);
}

#[test]
fn cli_can_inspect_an_individual_rendered_color() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/chair.json");
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_tessegrain"))
        .args(["inspect", path, "--color", "#96613d"])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(json["selection"]["rgb"], "#96613d");
    assert_eq!(json["visible_rgba_colors"], 1);
    assert!(json["visible_pixels"].as_u64().unwrap() > 0);
}

#[test]
fn repeated_runs_across_direction_changes_are_not_combined() {
    let mut image = RgbaImage::new(6, 3);
    for (x, y) in [0, 0, 1, 1, 0, 0].into_iter().enumerate() {
        image.put_pixel(x as u32, y, Rgba([1, 2, 3, 255]));
    }
    let result = inspect(&image, None).unwrap();
    assert!(result.edges[0].irregularities.is_empty());
    assert_ne!(result.edges[0].runs[1].chain, result.edges[0].runs[2].chain);
}

#[test]
fn partially_transparent_pixels_count_as_visible() {
    let mut image = RgbaImage::new(2, 1);
    image.put_pixel(0, 0, Rgba([1, 2, 3, 1]));
    image.put_pixel(1, 0, Rgba([4, 5, 6, 0]));
    let result = inspect(&image, None).unwrap();
    assert_eq!(
        (
            result.visible_pixels,
            result.partially_transparent_pixels,
            result.isolated_pixels
        ),
        (1, 1, 1)
    );
}

#[test]
fn cli_inspects_png_and_rejects_malformed_color() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/matching-couch/couch.png"
    );
    let bin = env!("CARGO_BIN_EXE_tessegrain");
    let valid = std::process::Command::new(bin)
        .args(["inspect", path])
        .output()
        .unwrap();
    assert!(valid.status.success());
    let json: serde_json::Value = serde_json::from_slice(&valid.stdout).unwrap();
    assert_eq!(json["canvas"]["width"], 60);
    let bad = std::process::Command::new(bin)
        .args(["inspect", path, "--color", "#gg0000"])
        .output()
        .unwrap();
    assert!(!bad.status.success());
}
