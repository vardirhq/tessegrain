//! An experimental hybrid pixel-art document and renderer.
pub mod inspect;
pub mod iso;
use image::{Rgba, RgbaImage};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    pub version: u32,
    pub canvas: Canvas,
    pub palette: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub materials: BTreeMap<String, iso::Material>,
    pub layers: Vec<Layer>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Canvas {
    pub width: u32,
    pub height: u32,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Layer {
    pub name: String,
    pub commands: Vec<Command>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Command {
    IsoScene {
        origin: [i32; 2],
        boxes: Vec<iso::IsoBox>,
        #[serde(default)]
        outline: bool,
    },
    Pixel {
        x: i32,
        y: i32,
        color: String,
    },
    Rect {
        x: i32,
        y: i32,
        w: u32,
        h: u32,
        color: String,
    },
    Polygon {
        points: Vec<[i32; 2]>,
        color: String,
    },
    Line {
        from: [i32; 2],
        to: [i32; 2],
        color: String,
    },
    Grid {
        x: i32,
        y: i32,
        colors: BTreeMap<char, String>,
        rows: Vec<String>,
    },
}
fn parse_color(s: &str) -> Result<Rgba<u8>, String> {
    let s = s.strip_prefix('#').ok_or("color must start with #")?;
    if !s.is_ascii() || (s.len() != 6 && s.len() != 8) {
        return Err("color must be #RRGGBB or #RRGGBBAA".into());
    }
    let mut out = [0, 0, 0, 255];
    for (i, byte) in out.iter_mut().enumerate().take(s.len() / 2) {
        *byte = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).map_err(|_| "invalid hex color")?;
    }
    Ok(Rgba(out))
}
fn put(im: &mut RgbaImage, x: i64, y: i64, c: Rgba<u8>) {
    if x >= 0 && y >= 0 && x < i64::from(im.width()) && y < i64::from(im.height()) {
        im.put_pixel(x as u32, y as u32, c);
    }
}
pub fn render(doc: &Document) -> Result<RgbaImage, String> {
    if doc.version != 1 {
        return Err("unsupported document version".into());
    }
    if doc.canvas.width == 0
        || doc.canvas.height == 0
        || doc.canvas.width > 2048
        || doc.canvas.height > 2048
    {
        return Err("canvas dimensions must be between 1 and 2048".into());
    }
    let palette: BTreeMap<_, _> = doc
        .palette
        .iter()
        .map(|(k, v)| Ok((k.as_str(), parse_color(v)?)))
        .collect::<Result<_, String>>()?;
    let get = |name: &str| {
        palette
            .get(name)
            .copied()
            .ok_or_else(|| format!("unknown palette color: {name}"))
    };
    let mut im = RgbaImage::new(doc.canvas.width, doc.canvas.height);
    for layer in &doc.layers {
        for cmd in &layer.commands {
            match cmd {
                Command::IsoScene {
                    origin,
                    boxes,
                    outline,
                } => {
                    iso::draw_scene(&mut im, *origin, boxes, &doc.materials, &palette, *outline)?;
                }
                Command::Pixel { x, y, color } => {
                    put(&mut im, i64::from(*x), i64::from(*y), get(color)?)
                }
                Command::Rect { x, y, w, h, color } => {
                    if *w == 0 || *h == 0 {
                        return Err("rectangle dimensions must be positive".into());
                    }
                    let c = get(color)?;
                    let x = i64::from(*x);
                    let y = i64::from(*y);
                    let x1 = x.max(0);
                    let y1 = y.max(0);
                    let x2 = (x + i64::from(*w)).min(i64::from(im.width()));
                    let y2 = (y + i64::from(*h)).min(i64::from(im.height()));
                    for py in y1..y2 {
                        for px in x1..x2 {
                            put(&mut im, px, py, c);
                        }
                    }
                }
                Command::Polygon { points, color } => {
                    if points.len() < 3 || points.len() > 256 {
                        return Err("polygon must have between 3 and 256 vertices".into());
                    }
                    let c = get(color)?;
                    let min_x = i64::from(points.iter().map(|p| p[0]).min().unwrap()).max(0);
                    let max_x = i64::from(points.iter().map(|p| p[0]).max().unwrap())
                        .min(i64::from(im.width()));
                    let min_y = i64::from(points.iter().map(|p| p[1]).min().unwrap()).max(0);
                    let max_y = i64::from(points.iter().map(|p| p[1]).max().unwrap())
                        .min(i64::from(im.height()));
                    for y in min_y..max_y {
                        for x in min_x..max_x {
                            let (cx, cy) = (i128::from(2 * x + 1), i128::from(2 * y + 1));
                            let mut inside = false;
                            for i in 0..points.len() {
                                let a = points[i];
                                let b = points[(i + 1) % points.len()];
                                let (ax, ay, bx, by) = (
                                    i128::from(a[0]) * 2,
                                    i128::from(a[1]) * 2,
                                    i128::from(b[0]) * 2,
                                    i128::from(b[1]) * 2,
                                );
                                if (ay > cy) != (by > cy) {
                                    let cross = (bx - ax) * (cy - ay) - (cx - ax) * (by - ay);
                                    if (by > ay && cross > 0) || (by < ay && cross < 0) {
                                        inside = !inside;
                                    }
                                }
                            }
                            if inside {
                                put(&mut im, x, y, c);
                            }
                        }
                    }
                }
                Command::Line { from, to, color } => {
                    let c = get(color)?;
                    let (mut x, mut y) = (i64::from(from[0]), i64::from(from[1]));
                    let (x2, y2) = (i64::from(to[0]), i64::from(to[1]));
                    let dx = (x2 - x).abs();
                    let dy = (y2 - y).abs();
                    if dx.max(dy) > 1_000_000 {
                        return Err("line too long".into());
                    }
                    let (sx, sy) = (if x < x2 { 1 } else { -1 }, if y < y2 { 1 } else { -1 });
                    let mut err = dx - dy;
                    loop {
                        put(&mut im, x, y, c);
                        if x == x2 && y == y2 {
                            break;
                        }
                        let e = 2 * err;
                        if e > -dy {
                            err -= dy;
                            x += sx
                        }
                        if e < dx {
                            err += dx;
                            y += sy
                        }
                    }
                }
                Command::Grid { x, y, colors, rows } => {
                    let width = rows.first().ok_or("grid must have rows")?.chars().count();
                    if width == 0 || rows.iter().any(|r| r.chars().count() != width) {
                        return Err("grid rows must have equal nonzero widths".into());
                    }
                    for (dy, row) in rows.iter().enumerate() {
                        for (dx, symbol) in row.chars().enumerate() {
                            let name = colors
                                .get(&symbol)
                                .ok_or_else(|| format!("grid symbol {symbol:?} is undefined"))?;
                            if name != "transparent" {
                                put(
                                    &mut im,
                                    i64::from(*x) + dx as i64,
                                    i64::from(*y) + dy as i64,
                                    get(name)?,
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(im)
}
pub fn import_image(im: &RgbaImage) -> Result<Document, String> {
    if im.width() == 0 || im.height() == 0 || im.width() > 2048 || im.height() > 2048 {
        return Err("canvas dimensions must be between 1 and 2048".into());
    }
    let mut palette = BTreeMap::new();
    let mut colors = BTreeMap::new();
    let mut lookup = BTreeMap::new();
    let mut rows = Vec::new();
    for y in 0..im.height() {
        let mut row = String::new();
        for x in 0..im.width() {
            let rgba = im.get_pixel(x, y).0;
            let c = if let Some(c) = lookup.get(&rgba) {
                *c
            } else {
                let index = lookup.len();
                let code = 0x1000u32
                    .checked_add(index as u32)
                    .ok_or("too many colors")?;
                let c = char::from_u32(code)
                    .filter(|c| !c.is_control())
                    .ok_or("too many colors")?;
                let name = format!("color{index}");
                palette.insert(
                    name.clone(),
                    format!(
                        "#{:02X}{:02X}{:02X}{:02X}",
                        rgba[0], rgba[1], rgba[2], rgba[3]
                    ),
                );
                colors.insert(c, name);
                lookup.insert(rgba, c);
                c
            };
            row.push(c);
        }
        rows.push(row);
    }
    Ok(Document {
        version: 1,
        canvas: Canvas {
            width: im.width(),
            height: im.height(),
        },
        palette,
        materials: BTreeMap::new(),
        layers: vec![Layer {
            name: "imported pixels".into(),
            commands: vec![Command::Grid {
                x: 0,
                y: 0,
                colors,
                rows,
            }],
        }],
    })
}
