"""Identity comparison for zero-key IBus browser setup observations."""


def same_engine_focus_identity(before, after):
    """Packet headers identify observations; client and engine path identify focus."""
    if not isinstance(before, dict) or not isinstance(after, dict):
        return False
    return all(
        isinstance(before.get(key), str)
        and bool(before[key])
        and before[key] == after.get(key)
        for key in ("client", "engine_object_path")
    )
