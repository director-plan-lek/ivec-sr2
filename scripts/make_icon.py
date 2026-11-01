def main() -> None:
    from PIL import Image, ImageDraw, ImageFilter

    size = 1024
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # rounded-square background with a soft vertical gradient
    pad = 64
    radius = 224
    for y in range(pad, size - pad):
        t = (y - pad) / (size - 2 * pad)
        r = int(18 + t * 24)
        g = int(26 + t * 44)
        b = int(48 + t * 120)
        draw.line([(pad, y), (size - pad, y)], fill=(r, g, b, 255))

    mask = Image.new("L", (size, size), 0)
    ImageDraw.Draw(mask).rounded_rectangle(
        [pad, pad, size - pad, size - pad], radius=radius, fill=255
    )
    img.putalpha(mask)

    # the "A" glyph
    glyph = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    gd = ImageDraw.Draw(glyph)
    stroke = 74
    left, right = 330, 694
    top, bottom = 300, 726
    gd.line([(left, bottom), (512, top)], fill=(255, 255, 255, 255), width=stroke)
    gd.line([(512, top), (right, bottom)], fill=(255, 255, 255, 255), width=stroke)
    gd.line([(left + 74, bottom - 118), (right - 74, bottom - 118)], fill=(255, 255, 255, 255), width=stroke)

    glow = glyph.filter(ImageFilter.GaussianBlur(26))
    img = Image.alpha_composite(img, glow)
    img = Image.alpha_composite(img, glyph)

    img.save("app-icon.png")
    print("wrote app-icon.png")


if __name__ == "__main__":
    main()
