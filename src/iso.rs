//! Isometric box scenes: deterministic 2:1 projection and per-pixel depth testing.
use image::{Rgba, RgbaImage};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Material {
    pub top: String,
    pub left: String,
    pub right: String,
    pub outline: String,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Shape {
    #[default]
    Box,
    Cylinder,
    StairsX,
    StairsY,
}
fn is_box(shape: &Shape) -> bool {
    *shape == Shape::Box
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IsoBox {
    #[serde(default, skip_serializing_if = "is_box")]
    pub shape: Shape,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub steps: Option<u32>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub reverse: bool,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub w: u32,
    pub d: u32,
    pub h: u32,
    pub material: String,
}
#[derive(Clone, Copy)]
struct Sample {
    depth: i64,
    color: Rgba<u8>,
    outline: Rgba<u8>,
}

fn point(origin: [i32; 2], x: i64, y: i64, z: i64) -> (i64, i64) {
    (
        i64::from(origin[0]) + 2 * (x - y),
        i64::from(origin[1]) + x + y - 2 * z,
    )
}
fn contains(vertices: &[(i64, i64); 4], x: i64, y: i64) -> bool {
    let (cx, cy) = (2 * x + 1, 2 * y + 1);
    let mut inside = false;
    for i in 0..4 {
        let (ax, ay) = vertices[i];
        let (bx, by) = vertices[(i + 1) % 4];
        let (ax, ay, bx, by) = (2 * ax, 2 * ay, 2 * bx, 2 * by);
        if (ay > cy) != (by > cy) {
            let cross = (bx - ax) * (cy - ay) - (cx - ax) * (by - ay);
            if (by > ay && cross > 0) || (by < ay && cross < 0) {
                inside = !inside
            }
        }
    }
    inside
}
fn raster(
    samples: &mut [Option<Sample>],
    dims: (u32, u32),
    origin: [i32; 2],
    vertices: [(i64, i64); 4],
    face: u8,
    fixed: i64,
    colors: (Rgba<u8>, Rgba<u8>),
) {
    let (color, outline) = colors;
    let (w, h) = dims;
    let x0 = vertices.iter().map(|p| p.0).min().unwrap().max(0);
    let x1 = vertices
        .iter()
        .map(|p| p.0)
        .max()
        .unwrap()
        .min(i64::from(w));
    let y0 = vertices.iter().map(|p| p.1).min().unwrap().max(0);
    let y1 = vertices
        .iter()
        .map(|p| p.1)
        .max()
        .unwrap()
        .min(i64::from(h));
    for y in y0..y1 {
        for x in x0..x1 {
            if !contains(&vertices, x, y) {
                continue;
            }
            let sx = 2 * (x - i64::from(origin[0])) + 1;
            let sy = 2 * (y - i64::from(origin[1])) + 1;
            let depth = match face {
                0 => 4 * sy + 24 * fixed,
                1 => 24 * fixed - 3 * sx - 2 * sy,
                _ => 24 * fixed + 3 * sx - 2 * sy,
            };
            let index = y as usize * w as usize + x as usize;
            let sample = Sample {
                depth,
                color,
                outline,
            };
            if samples[index].is_none_or(|p| {
                (sample.depth, sample.color.0, sample.outline.0) > (p.depth, p.color.0, p.outline.0)
            }) {
                samples[index] = Some(sample)
            }
        }
    }
}
/// All boxes in one scene share a depth buffer. Later 2D commands retain normal draw order.
pub fn draw_scene(
    image: &mut RgbaImage,
    origin: [i32; 2],
    boxes: &[IsoBox],
    materials: &BTreeMap<String, Material>,
    palette: &BTreeMap<&str, Rgba<u8>>,
    outline: bool,
) -> Result<(), String> {
    if boxes.len() > 512 {
        return Err("iso_scene supports at most 512 boxes".into());
    }
    if origin.iter().any(|n| n.unsigned_abs() > 1_000_000) {
        return Err("iso_scene origin outside supported range".into());
    }
    let mut expanded = Vec::new();
    for b in boxes {
        if [b.w, b.d, b.h].iter().any(|&n| n == 0 || n > 1024) {
            return Err("box dimensions must be between 1 and 1024".into());
        }
        if [b.x, b.y, b.z].iter().any(|n| n.unsigned_abs() > 4096) {
            return Err("box coordinates must be between -4096 and 4096".into());
        }
        if !matches!(b.shape, Shape::StairsX | Shape::StairsY) && b.steps.is_some() {
            return Err("steps is only valid for stairs".into());
        }
        if b.reverse && !matches!(b.shape, Shape::StairsX | Shape::StairsY) {
            return Err("reverse is only valid for stairs".into());
        }
        match b.shape {
            Shape::Box => expanded.push(b.clone()),
            Shape::Cylinder => {
                if b.w > 64 || b.d > 64 {
                    return Err("cylinder width and depth must be at most 64".into());
                }
                let (w, d) = (i64::from(b.w), i64::from(b.d));
                for row in 0..b.d {
                    let dy = 2 * i64::from(row) + 1 - d;
                    let mut start = None;
                    for col in 0..b.w {
                        let dx = 2 * i64::from(col) + 1 - w;
                        if dx * dx * d * d + dy * dy * w * w <= w * w * d * d {
                            start.get_or_insert(col);
                        } else if let Some(first) = start.take() {
                            let mut part = b.clone();
                            part.x += first as i32;
                            part.y += row as i32;
                            part.w = col - first;
                            part.d = 1;
                            part.shape = Shape::Box;
                            expanded.push(part);
                        }
                    }
                    if let Some(first) = start {
                        let mut part = b.clone();
                        part.x += first as i32;
                        part.y += row as i32;
                        part.w = b.w - first;
                        part.d = 1;
                        part.shape = Shape::Box;
                        expanded.push(part);
                    }
                }
            }
            Shape::StairsX | Shape::StairsY => {
                let length = if b.shape == Shape::StairsX { b.w } else { b.d };
                let steps = b.steps.ok_or("stairs require steps")?;
                if steps == 0 || steps > 64 || steps > length || steps > b.h {
                    return Err("stairs steps must be 1..=min(64, length, height)".into());
                }
                for i in 0..steps {
                    let start = i * length / steps;
                    let end = (i + 1) * length / steps;
                    let mut part = b.clone();
                    part.shape = Shape::Box;
                    part.steps = None;
                    part.h = if b.reverse {
                        (steps - i) * b.h / steps
                    } else {
                        (i + 1) * b.h / steps
                    };
                    if b.shape == Shape::StairsX {
                        part.x += start as i32;
                        part.w = end - start
                    } else {
                        part.y += start as i32;
                        part.d = end - start
                    }
                    expanded.push(part);
                }
            }
        }
        if expanded.len() > 4096 {
            return Err("iso_scene exceeds 4096 expanded box sections".into());
        }
    }
    let (width, height) = image.dimensions();
    let mut pixels = vec![None; width as usize * height as usize];
    for b in &expanded {
        let m = materials
            .get(&b.material)
            .ok_or_else(|| format!("unknown material: {}", b.material))?;
        let color = |s: &str| -> Result<Rgba<u8>, String> {
            let p = palette
                .get(s)
                .copied()
                .ok_or_else(|| format!("unknown palette color: {s}"))?;
            if p.0[3] != 255 {
                return Err("isometric material colors must be opaque".into());
            }
            Ok(p)
        };
        let (top, left, right, edge) = (
            color(&m.top)?,
            color(&m.left)?,
            color(&m.right)?,
            color(&m.outline)?,
        );
        let (x, y, z) = (i64::from(b.x), i64::from(b.y), i64::from(b.z));
        let (x1, y1, z1) = (x + i64::from(b.w), y + i64::from(b.d), z + i64::from(b.h));
        let a = point(origin, x, y, z1);
        let bb = point(origin, x1, y, z1);
        let c = point(origin, x1, y1, z1);
        let d = point(origin, x, y1, z1);
        let b0 = point(origin, x1, y, z);
        let c0 = point(origin, x1, y1, z);
        let d0 = point(origin, x, y1, z);
        raster(
            &mut pixels,
            (width, height),
            origin,
            [a, bb, c, d],
            0,
            z1,
            (top, edge),
        );
        raster(
            &mut pixels,
            (width, height),
            origin,
            [bb, c, c0, b0],
            1,
            x1,
            (left, edge),
        );
        raster(
            &mut pixels,
            (width, height),
            origin,
            [d, c, c0, d0],
            2,
            y1,
            (right, edge),
        );
    }
    for y in 0..height {
        for x in 0..width {
            let i = (y * width + x) as usize;
            if let Some(s) = pixels[i] {
                let boundary = outline
                    && (x == 0
                        || y == 0
                        || x + 1 == width
                        || y + 1 == height
                        || pixels[i - 1].is_none()
                        || pixels[i + 1].is_none()
                        || pixels[i - width as usize].is_none()
                        || pixels[i + width as usize].is_none());
                image.put_pixel(x, y, if boundary { s.outline } else { s.color });
            }
        }
    }
    Ok(())
}
