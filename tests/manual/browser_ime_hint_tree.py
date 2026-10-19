"""Read-only, complete visible AT-SPI subtree walk for a native test observer."""


class IncompleteHintObservation(RuntimeError):
    """The observer cannot establish absence or uniqueness of a visible hint."""


def visible_nodes(root, state_type, *, max_nodes=1000, max_edges=4000, max_depth=12):
    """Return (accessible, index path) pairs or refuse incomplete evidence.

    Only the application root is exempt from visibility pruning. Object identity
    is its AT-SPI bus name/path, never a caption or Python proxy address. No
    partially traversed result is returned after a budget or identity failure.
    """
    def identity(node):
        bus, path = node.app.bus_name, node.path
        if not isinstance(bus, str) or not bus or not isinstance(path, str) or not path:
            raise IncompleteHintObservation("accessible object identity unavailable")
        return bus, path

    owner_bus = identity(root)[0]
    seen, result, todo, edges = set(), [], [(root, [0], 0)], 0
    while todo:
        node, path, depth = todo.pop()
        key = identity(node)
        if key[0] != owner_bus:
            raise IncompleteHintObservation("foreign application in owned accessible subtree")
        if key in seen:
            continue
        if len(seen) >= max_nodes:
            raise IncompleteHintObservation("bounded visible accessible node budget exceeded")
        seen.add(key)
        node.clear_cache_single()
        state = node.get_state_set()
        if state.contains(state_type.DEFUNCT):
            raise IncompleteHintObservation("owned accessible object is defunct")
        if depth and not (state.contains(state_type.SHOWING)
                          and state.contains(state_type.VISIBLE)):
            continue
        result.append((node, path))
        count = node.get_child_count()
        if type(count) is not int or count < 0:
            raise IncompleteHintObservation("accessible child count unavailable")
        if count and depth >= max_depth:
            raise IncompleteHintObservation("bounded visible accessible depth exceeded")
        if count > max_edges - edges:
            raise IncompleteHintObservation("bounded visible accessible edge budget exceeded")
        edges += count
        children = []
        for index in range(count):
            child = node.get_child_at_index(index)
            if child is None:
                raise IncompleteHintObservation("accessible child became unavailable")
            children.append((child, path + [index], depth + 1))
        todo.extend(reversed(children))
    return result
