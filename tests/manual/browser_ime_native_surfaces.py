"""Native test observations; these helpers never authorize runtime edits."""


def matches_typed_seed(raw_text, caret, selections, expected, *, rich_text=False):
    """Match the whole seed, allowing a rich editor's representation of Space."""
    if (
        not isinstance(raw_text, str)
        or not isinstance(expected, str)
        or type(caret) is not int
        or type(selections) is not int
        or type(rich_text) is not bool
    ):
        return False
    if caret != len(raw_text) or selections != 0 or len(raw_text) != len(expected):
        return False
    return all(
        actual == wanted or (rich_text and wanted == " " and actual == "\u00a0")
        for actual, wanted in zip(raw_text, expected)
    )


def native_dom_text_state(snapshot, *, expected_url, expected_field, expected_type):
    """Validate one fresh read from a separately pinned, read-only DOM route.

    The caller verifies process/transport identity around each read and retains
    the native focus/layout/engine checks. This function does not choose a
    reader or establish that an IME composition has committed.
    """
    if not isinstance(snapshot, dict) or any(
        not isinstance(value, str) or not value
        for value in (expected_url, expected_field, expected_type)
    ):
        raise ValueError("invalid DOM observation contract")
    if (
        snapshot.get("url") != expected_url
        or snapshot.get("field_id") != expected_field
        or snapshot.get("active_id") != expected_field
        or snapshot.get("type") != expected_type
        or snapshot.get("document_focused") is not True
    ):
        raise ValueError("DOM field identity or focus mismatch")
    value = snapshot.get("value")
    start = snapshot.get("selection_start")
    end = snapshot.get("selection_end")
    direction = snapshot.get("selection_direction")
    if (
        not isinstance(value, str)
        or len(value) > 2048
        or type(start) is not int
        or type(end) is not int
        or start < 0
        or end < start
        or direction not in ("none", "forward", "backward")
    ):
        raise ValueError("invalid DOM text or selection")
    # DOM offsets count UTF-16 units; the native actor counts Unicode scalars.
    offsets = {0: 0}
    units = 0
    for index, character in enumerate(value, 1):
        scalar = ord(character)
        if 0xD800 <= scalar <= 0xDFFF:
            raise ValueError("unpaired surrogate in DOM text")
        units += 2 if scalar > 0xFFFF else 1
        offsets[units] = index
    if start not in offsets or end not in offsets:
        raise ValueError("DOM selection outside text or inside a surrogate pair")
    return {
        "committed": value,
        "caret": offsets[start if direction == "backward" else end],
        "selection": start != end,
        "selection_start": offsets[start],
        "selection_end": offsets[end],
        "focused": True,
        "observer": "pinned read-only client DOM text",
    }
