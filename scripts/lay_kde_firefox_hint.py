"""KDE Firefox suffix typography. Receives display data; never handles input."""


def render_suffix(label, style, available: int, height: int) -> None:
    try:
        from PyQt6.QtCore import Qt
        from PyQt6.QtGui import QColor, QFontMetrics
    except ImportError:
        from PyQt5.QtCore import Qt
        from PyQt5.QtGui import QColor, QFontMetrics

    font = label.font()
    if style.get("font_family"):
        font.setFamily(style["font_family"])
    # Native character bounds carry actual device-pixel line metrics. AT-SPI
    # point sizes are rounded, so fit the font's line height instead of using
    # a fixed fraction or an assumed CSS font size.
    low, high = 1, height
    while low < high:
        middle = (low + high + 1) // 2
        font.setPixelSize(middle)
        if QFontMetrics(font).height() <= height:
            low = middle
        else:
            high = middle - 1
    font.setPixelSize(low)
    label.setFont(font)
    label.setStyleSheet(
        "color: #888888; background: #ffffff; border: none; margin: 0; padding: 0;"
    )
    label.resize(max(1, min(available, QFontMetrics(font).horizontalAdvance(label.text()))), height)
    # Reuse the previously source-proved XShape path: only suffix glyphs
    # remain visible even when the KDE compositor is disabled. No opacity.
    label.clearMask()
    label.setMask(label.grab().createMaskFromColor(
        QColor("#ffffff"), getattr(Qt, "MaskMode", Qt).MaskInColor
    ))
