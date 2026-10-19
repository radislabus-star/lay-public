import importlib.util
from pathlib import Path
import unittest
from types import SimpleNamespace

PATH = Path(__file__).resolve().parents[2] / "tests/manual/browser_ime_hint_tree.py"
SPEC = importlib.util.spec_from_file_location("browser_ime_hint_tree", PATH)
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)
IncompleteHintObservation = MODULE.IncompleteHintObservation
visible_nodes = MODULE.visible_nodes


class States:
    SHOWING = "showing"
    VISIBLE = "visible"
    DEFUNCT = "defunct"


class Node:
    def __init__(self, path, children=(), *, visible=True, showing=True,
                 bus=":1.9", defunct=False):
        self.path, self.app, self.children = path, SimpleNamespace(bus_name=bus), list(children)
        self.states = {key for key, enabled in (
            (States.SHOWING, showing), (States.VISIBLE, visible), (States.DEFUNCT, defunct)
        ) if enabled}
        self.child_reads = 0

    def clear_cache_single(self):
        pass

    def get_state_set(self):
        return SimpleNamespace(contains=lambda state: state in self.states)

    def get_child_count(self):
        self.child_reads += 1
        return len(self.children)

    def get_child_at_index(self, index):
        self.child_reads += 1
        return self.children[index]


class VisibleTreeTests(unittest.TestCase):
    def test_missing_child_refuses_instead_of_authorizing_partial_evidence(self):
        for children in ([Node('/hint'), None], [None]):
            with self.subTest(children=len(children)):
                with self.assertRaisesRegex(IncompleteHintObservation, 'child became unavailable'):
                    visible_nodes(Node('/root', children), States)

    def test_hidden_subtree_is_not_enumerated(self):
        hidden = Node("/hidden", [Node(f"/old/{i}") for i in range(1500)], showing=False)
        hint = Node("/hint")
        root = Node("/root", [hidden, hint], visible=False, showing=False)
        found = visible_nodes(root, States)
        self.assertEqual([(node.path, path) for node, path in found],
                         [("/root", [0]), ("/hint", [0, 1])])
        self.assertEqual(hidden.child_reads, 0)

    def test_each_visibility_flag_prunes_nonroot(self):
        for flags in ({"visible": False}, {"showing": False}):
            with self.subTest(flags=flags):
                hidden = Node("/hidden", [Node("/hidden/hint")], **flags)
                self.assertEqual(len(visible_nodes(Node("/root", [hidden]), States)), 1)
                self.assertEqual(hidden.child_reads, 0)

    def test_cycles_and_distinct_proxies_for_same_object_are_visited_once(self):
        proxy = Node("/root")
        shared = Node("/label")
        root = Node("/root", [proxy, shared, Node("/label")])
        proxy.children = [root]
        found = visible_nodes(root, States)
        self.assertEqual([node.path for node, _ in found], ["/root", "/label"])
        self.assertEqual(proxy.child_reads, 0)

    def test_two_distinct_visible_labels_remain_ambiguous_to_consumer(self):
        found = visible_nodes(Node("/root", [Node("/label/a"), Node("/label/b")]), States)
        self.assertEqual([path for _, path in found], [[0], [0, 0], [0, 1]])

    def test_node_budget_refuses_instead_of_returning_partial_evidence(self):
        with self.assertRaisesRegex(RuntimeError, "node budget"):
            visible_nodes(Node("/root", [Node("/a"), Node("/b")]), States, max_nodes=2)

    def test_edge_budget_refuses_before_fanout_is_enumerated(self):
        root = Node("/root", [Node("/a"), Node("/b")])
        with self.assertRaisesRegex(RuntimeError, "edge budget"):
            visible_nodes(root, States, max_edges=1)
        self.assertEqual(root.child_reads, 1)

    def test_depth_budget_refuses_visible_truncation(self):
        root = Node("/root", [Node("/panel", [Node("/hidden-second-label")])])
        with self.assertRaisesRegex(RuntimeError, "depth"):
            visible_nodes(root, States, max_depth=1)

    def test_foreign_bus_and_absent_identity_refuse(self):
        for node in (Node("/label", bus=":1.10"), Node(""), Node("/label", bus="")):
            with self.subTest(path=node.path, bus=node.app.bus_name):
                with self.assertRaisesRegex(RuntimeError, "foreign|identity"):
                    visible_nodes(Node("/root", [node]), States)

    def test_defunct_root_or_child_refuses(self):
        for root in (Node("/root", defunct=True),
                     Node("/root", [Node("/child", defunct=True)])):
            with self.assertRaisesRegex(RuntimeError, "defunct"):
                visible_nodes(root, States)


if __name__ == "__main__":
    unittest.main()
