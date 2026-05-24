#!/usr/bin/env python3
"""Render a small demo GIF for README/articles."""

from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw, ImageFont


ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs" / "publicity" / "demo.gif"
FONT_MONO = "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf"
FONT_SANS = "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"


def font(path: str, size: int) -> ImageFont.FreeTypeFont:
    return ImageFont.truetype(path, size)


TITLE = font(FONT_SANS, 34)
TEXT = font(FONT_SANS, 25)
MONO = font(FONT_MONO, 34)
MONO_SMALL = font(FONT_MONO, 26)


def draw_frame(line: str, status: str, highlight: str | None = None) -> Image.Image:
    image = Image.new("RGB", (1200, 675), "#101418")
    draw = ImageDraw.Draw(image)

    draw.rounded_rectangle((50, 42, 1150, 633), radius=26, fill="#171d24", outline="#2c3845", width=2)
    draw.text((80, 72), "lay", font=TITLE, fill="#f2f5f8")
    draw.text((150, 82), "double Shift layout rescue", font=TEXT, fill="#9fb0c3")

    draw.rounded_rectangle((80, 150, 1120, 285), radius=18, fill="#0d1117", outline="#334252", width=2)
    draw.text((115, 190), line, font=MONO, fill="#eef3f7")

    if highlight:
        x0 = 115 + len(line.replace(highlight, "")) * 20
        draw.text((115, 190), line.replace(highlight, ""), font=MONO, fill="#eef3f7")
        draw.text((x0, 190), highlight, font=MONO, fill="#70d6ff")

    draw.rounded_rectangle((80, 330, 1120, 430), radius=18, fill="#1d2a35", outline="#3c5367", width=1)
    draw.text((115, 362), status, font=MONO_SMALL, fill="#d9e7f2")

    draw.text((80, 515), "Double Shift: исправить раскладку прямо в приложении", font=TEXT, fill="#d7dee7")
    draw.text((80, 555), "RU/EN · GNOME/KDE/X11 · локально · без облака", font=TEXT, fill="#91a2b5")

    return image


def main() -> None:
    frames: list[Image.Image] = []
    durations: list[int] = []

    scenes = [
        ("ghbdtn", "Shift Shift -> привет", "привет"),
        ("good ntrcn", "Shift Shift -> good текст", "good текст"),
        ("перпаратов ", "Space -> препаратов", "препаратов "),
    ]

    for typed, action, result in scenes:
        for idx in range(1, len(typed) + 1):
            frames.append(draw_frame(typed[:idx], "печатаю..."))
            durations.append(80)
        frames.append(draw_frame(typed, action))
        durations.append(700)
        frames.append(draw_frame(result, "готово", result))
        durations.append(1100)
        frames.append(draw_frame("", ""))
        durations.append(250)

    OUT.parent.mkdir(parents=True, exist_ok=True)
    frames[0].save(
        OUT,
        save_all=True,
        append_images=frames[1:],
        duration=durations,
        loop=0,
        optimize=True,
    )
    print(f"wrote {OUT}")


if __name__ == "__main__":
    main()
