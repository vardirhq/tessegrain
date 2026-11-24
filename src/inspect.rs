//! Objective observations of rendered pixels. Observations are not quality scores.
use image::RgbaImage;
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Region {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
#[derive(Debug, Serialize)]
pub struct Run {
    pub x_start: u32,
    pub x_end: u32,
    pub y: u32,
    pub length: u32,
    pub chain: u32,
}
#[derive(Debug, Serialize)]
pub struct Irregularity {
    pub x_start: u32,
    pub x_end: u32,
    pub y: u32,
    pub length: u32,
    pub neighboring_length: u32,
    pub note: &'static str,
}
#[derive(Debug, Serialize)]
pub struct EdgeReport {
    pub edge: &'static str,
    pub runs: Vec<Run>,
    pub irregularities: Vec<Irregularity>,
}
#[derive(Debug, Serialize)]
pub struct Canvas {
    pub width: u32,
    pub height: u32,
}
#[derive(Debug, Serialize)]
pub struct Report {
    pub canvas: Canvas,
    pub region: Region,
    pub visible_pixels: u64,
    pub partially_transparent_pixels: u64,
    pub visible_rgba_colors: usize,
    pub isolated_pixels: u64,
    pub edges: Vec<EdgeReport>,
}

/// Inspect the requested rectangle, using alpha > 0 as visible and 8-neighbor isolation.
/// Top and bottom contours are per-column outermost visible pixels within the region;
/// holes, internal color boundaries, and other contours are not traced.
pub fn inspect(image: &RgbaImage, region: Option<Region>) -> Result<Report, String> {
    let region = region.unwrap_or(Region {
        x: 0,
        y: 0,
        width: image.width(),
        height: image.height(),
    });
    if region.width == 0
        || region.height == 0
        || region
            .x
            .checked_add(region.width)
            .is_none_or(|x| x > image.width())
        || region
            .y
            .checked_add(region.height)
            .is_none_or(|y| y > image.height())
    {
        return Err("region must be nonempty and entirely inside the canvas".into());
    }
    let (x_end, y_end) = (region.x + region.width, region.y + region.height);
    let mut colors = BTreeSet::new();
    let mut visible = 0;
    let mut partial = 0;
    let mut isolated = 0;
    for y in region.y..y_end {
        for x in region.x..x_end {
            let rgba = image.get_pixel(x, y).0;
            if rgba[3] == 0 {
                continue;
            }
            visible += 1;
            partial += u64::from(rgba[3] < 255);
            colors.insert(rgba);
            let has_neighbor = (-1i64..=1).any(|dy| {
                (-1i64..=1).any(|dx| {
                    if dx == 0 && dy == 0 {
                        return false;
                    }
                    let nx = i64::from(x) + dx;
                    let ny = i64::from(y) + dy;
                    nx >= i64::from(region.x)
                        && nx < i64::from(x_end)
                        && ny >= i64::from(region.y)
                        && ny < i64::from(y_end)
                        && image.get_pixel(nx as u32, ny as u32).0[3] > 0
                })
            });
            if !has_neighbor {
                isolated += 1
            }
        }
    }
    let mut upper = Vec::new();
    let mut lower = Vec::new();
    for x in region.x..x_end {
        let top = (region.y..y_end).find(|&y| image.get_pixel(x, y).0[3] > 0);
        let bottom = (region.y..y_end)
            .rev()
            .find(|&y| image.get_pixel(x, y).0[3] > 0);
        upper.push((x, top));
        lower.push((x, bottom));
    }
    Ok(Report {
        canvas: Canvas {
            width: image.width(),
            height: image.height(),
        },
        region,
        visible_pixels: visible,
        partially_transparent_pixels: partial,
        visible_rgba_colors: colors.len(),
        isolated_pixels: isolated,
        edges: vec![edge_report("top", &upper), edge_report("bottom", &lower)],
    })
}
fn edge_report(edge: &'static str, columns: &[(u32, Option<u32>)]) -> EdgeReport {
    let mut runs: Vec<Run> = Vec::new();
    let mut chain = 0;
    let mut previous: Option<(u32, u32)> = None;
    let mut direction = 0i64;
    for &(x, y) in columns {
        let Some(y) = y else {
            previous = None;
            direction = 0;
            continue;
        };
        if let Some((px, py)) = previous {
            let step = i64::from(y) - i64::from(py);
            let sign = step.signum();
            if x != px + 1 || step.abs() > 1 || (sign != 0 && direction != 0 && sign != direction) {
                chain += 1;
                direction = 0;
            }
            if sign != 0 {
                direction = sign;
            }
            if step == 0 && x == px + 1 && runs.last().is_some_and(|r| r.chain == chain && r.y == y)
            {
                let r = runs.last_mut().unwrap();
                r.x_end = x;
                r.length += 1;
                previous = Some((x, y));
                continue;
            }
        } else if !runs.is_empty() {
            chain += 1;
        }
        runs.push(Run {
            x_start: x,
            x_end: x,
            y,
            length: 1,
            chain,
        });
        previous = Some((x, y));
    }
    let mut irregularities = Vec::new();
    for triplet in runs.windows(3) {
        let (a, b, c) = (&triplet[0], &triplet[1], &triplet[2]);
        if a.chain == b.chain
            && b.chain == c.chain
            && a.length == c.length
            && b.length != a.length
            && a.y != b.y
            && b.y != c.y
        {
            irregularities.push(Irregularity {
                x_start: b.x_start,
                x_end: b.x_end,
                y: b.y,
                length: b.length,
                neighboring_length: a.length,
                note: "Run length differs from equal neighboring runs; inspect visually.",
            });
        }
    }
    EdgeReport {
        edge,
        runs,
        irregularities,
    }
}
