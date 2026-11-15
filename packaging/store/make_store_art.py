"""Generates the Microsoft Store listing art (box art, poster and hero image).

Usage (from the repository root, needs Pillow: pip install pillow):

    python packaging/store/make_store_art.py

The ship comes from packaging/store/art/ship.png (written by
`nebulance --export-store-assets <folder>`), the title font is the game's own
Press Start 2P. Output goes to packaging/store/art/.
"""

import colorsys
import os
import random

from PIL import Image, ImageDraw, ImageFilter, ImageFont

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
ART = os.path.join(ROOT, "packaging", "store", "art")
FONT = os.path.join(ROOT, "assets", "fonts", "PressStart2P.ttf")
BG = (24, 18, 48)
TITLE = "NEBULANCE"


def hsl(h, s, l):
    r, g, b = colorsys.hls_to_rgb(h % 1.0, l, s)
    return (int(r * 255), int(g * 255), int(b * 255))


def space(w, h, seed):
    """Dark sky with soft nebula clouds and stars."""
    rnd = random.Random(seed)
    img = Image.new("RGB", (w, h), BG)
    clouds = Image.new("RGB", (w, h), (0, 0, 0))
    d = ImageDraw.Draw(clouds)
    for _ in range(14):
        r = rnd.uniform(0.15, 0.4) * max(w, h)
        x, y = rnd.uniform(0, w), rnd.uniform(0, h)
        c = hsl(rnd.uniform(0.62, 0.9), 0.8, rnd.uniform(0.12, 0.25))
        d.ellipse((x - r, y - r, x + r, y + r), fill=c)
    clouds = clouds.filter(ImageFilter.GaussianBlur(max(w, h) / 10))
    img = Image.blend(img, clouds, 0.55)
    d = ImageDraw.Draw(img)
    px = max(1, min(w, h) // 360)
    for _ in range(w * h // 2500):
        x, y = rnd.randrange(w), rnd.randrange(h)
        v = rnd.randint(120, 255)
        s = px * rnd.choice([1, 1, 1, 2])
        d.rectangle((x, y, x + s - 1, y + s - 1), fill=(v, v, min(255, v + 30)))
    return img


def ship(height):
    src = Image.open(os.path.join(ART, "ship.png")).convert("RGBA")
    k = max(1, height // src.height)
    return src.resize((src.width * k, src.height * k), Image.NEAREST)


def glow(img, sprite, pos, color, radius):
    halo = Image.new("RGBA", img.size, (0, 0, 0, 0))
    mask = sprite.split()[3]
    tinted = Image.new("RGBA", sprite.size, color + (255,))
    halo.paste(tinted, pos, mask)
    halo = halo.filter(ImageFilter.GaussianBlur(radius))
    img.alpha_composite(halo)


def title(img, cx, cy, max_w):
    """The logo: one hue per letter, like the title screen, with a dark outline."""
    size = 200
    font = ImageFont.truetype(FONT, size)
    while font.getlength(TITLE) > max_w and size > 8:
        size -= 2
        font = ImageFont.truetype(FONT, size)
    w = font.getlength(TITLE)
    x = cx - w / 2
    d = ImageDraw.Draw(img)
    o = max(2, size // 12)
    for i, ch in enumerate(TITLE):
        c = hsl(0.62 + i * 0.03, 0.9, 0.62)
        for dx in range(-o, o + 1, max(1, o // 2)):
            for dy in range(-o, o + 1, max(1, o // 2)):
                d.text((x + dx, cy + dy), ch, font=font, fill=(20, 8, 40), anchor="lm")
        d.text((x, cy), ch, font=font, fill=c, anchor="lm")
        x += font.getlength(ch)
    return size


def subtitle(img, text, cx, y, size):
    font = ImageFont.truetype(FONT, size)
    ImageDraw.Draw(img).text((cx, y), text, font=font, fill=(190, 200, 230), anchor="mm")


def make(w, h, name, title_y=None, ship_frac=0.42, ship_y=0.5, sub=None, seed=7):
    img = space(w, h, seed).convert("RGBA")
    sp = ship(int(h * ship_frac))
    pos = ((w - sp.width) // 2, int(h * ship_y - sp.height / 2))
    glow(img, sp, pos, (80, 160, 255), max(w, h) // 40)
    img.alpha_composite(sp, pos)
    if title_y is not None:
        size = title(img, w / 2, h * title_y, w * 0.86)
        if sub:
            subtitle(img, sub, w / 2, h * title_y + size * 0.95, max(10, size // 4))
    img.convert("RGB").save(os.path.join(ART, name), optimize=True)
    print("wrote", name, f"{w}x{h}")


if __name__ == "__main__":
    # 1:1 box art (Store tile and search results)
    make(2160, 2160, "box_art_2160x2160.png", title_y=0.2, ship_frac=0.5, ship_y=0.6,
         sub="a formation shoot 'em up")
    make(1080, 1080, "box_art_1080x1080.png", title_y=0.2, ship_frac=0.5, ship_y=0.6,
         sub="a formation shoot 'em up")
    # 2:3 poster art (main image on the product page for games)
    make(1440, 2160, "poster_art_1440x2160.png", title_y=0.16, ship_frac=0.36, ship_y=0.58,
         sub="a formation shoot 'em up")
    make(720, 1080, "poster_art_720x1080.png", title_y=0.16, ship_frac=0.36, ship_y=0.58,
         sub="a formation shoot 'em up")
    # 16:9 super hero art (top of the product page; Microsoft asks for no text on it)
    make(3840, 2160, "hero_art_3840x2160.png", ship_frac=0.5, ship_y=0.5, seed=11)
    make(1920, 1080, "hero_art_1920x1080.png", ship_frac=0.5, ship_y=0.5, seed=11)
