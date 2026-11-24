"""Compare visible pixels and create a nearest-neighbor benchmark preview.

Requires Pillow: python -m pip install Pillow
"""
from pathlib import Path
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent
reference = Image.open(ROOT / 'reference.png').convert('RGBA')
recreation = Image.open(ROOT / 'recreation.png').convert('RGBA')
if reference.size != recreation.size:
    raise SystemExit('Images must have the same dimensions')

intersection = union = matches = 0
for y in range(reference.height):
    for x in range(reference.width):
        a, b = reference.getpixel((x, y)), recreation.getpixel((x, y))
        in_a, in_b = a[3] != 0, b[3] != 0
        intersection += in_a and in_b
        union += in_a or in_b
        matches += (in_a or in_b) and a == b
print(f'Visible silhouette intersection over union: {intersection}/{union} = {intersection / union:.2%}' if union else 'Empty silhouettes')
print(f'Exact RGBA matches over visible union: {matches}/{union} = {matches / union:.2%}' if union else 'No visible pixels')

scale = 8
w, h = reference.width * scale, reference.height * scale
preview = Image.new('RGB', (w * 2 + 36, h + 44), '#dedede')
draw = ImageDraw.Draw(preview)
for index, (label, image) in enumerate((('Reference', reference), ('Tessegrain redraw', recreation))):
    x = 12 + index * (w + 12)
    draw.text((x, 8), label, fill='#1a1a1a')
    draw.rectangle((x, 28, x + w - 1, 28 + h - 1), fill='#fafafa')
    enlarged = image.resize((w, h), Image.Resampling.NEAREST)
    preview.paste(enlarged, (x, 28), enlarged)
preview.save(ROOT / 'comparison.png')
