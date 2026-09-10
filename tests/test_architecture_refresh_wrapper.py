#!/usr/bin/env python3
"""Functional fail-stop tests for the canonical architecture refresh wrapper."""

from __future__ import annotations

import os
import shutil
import subprocess
import tempfile
import textwrap
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
WRAPPER = ROOT / "scripts" / "update-architecture-graph.sh"


def write_executable(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(textwrap.dedent(content).lstrip(), encoding="utf-8")
    path.chmod(0o755)


class ArchitectureRefreshWrapperTest(unittest.TestCase):
    def run_wrapper(self, fail_step: str | None = None) -> tuple[int, list[str]]:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            scripts = root / "scripts"
            scripts.mkdir()
            wrapper = scripts / WRAPPER.name
            shutil.copy2(WRAPPER, wrapper)
            log = root / "steps.log"
            log.touch()

            write_executable(
                root / "bin" / "graphify",
                """
                #!/usr/bin/env python3
                import os
                import sys
                from pathlib import Path

                if sys.argv[1:] != ["update", "."]:
                    raise SystemExit(64)
                with Path(os.environ["LAY_TD105_LOG"]).open("a", encoding="utf-8") as handle:
                    handle.write("graphify\\n")
                if os.environ.get("LAY_TD105_FAIL_STEP") == "graphify":
                    raise SystemExit(17)
                """,
            )
            write_executable(
                scripts / "prune-graphify-self-sources.py",
                """
                #!/usr/bin/env python3
                import os
                import sys
                from pathlib import Path

                steps = {
                    (): "prune",
                    ("--check",): "prune-check",
                }
                step = steps.get(tuple(sys.argv[1:]))
                if step is None:
                    raise SystemExit(64)
                with Path(os.environ["LAY_TD105_LOG"]).open("a", encoding="utf-8") as handle:
                    handle.write(step + "\\n")
                if os.environ.get("LAY_TD105_FAIL_STEP") == step:
                    raise SystemExit(20)
                """,
            )
            write_executable(
                scripts / "architecture_graph_gate.py",
                """
                #!/usr/bin/env python3
                import os
                import sys
                from pathlib import Path

                steps = {
                    ("--write-graph-binding",): "binding",
                    ("--write-receipt",): "receipt",
                }
                step = steps.get(tuple(sys.argv[1:]))
                if step is None:
                    raise SystemExit(64)
                with Path(os.environ["LAY_TD105_LOG"]).open("a", encoding="utf-8") as handle:
                    handle.write(step + "\\n")
                if os.environ.get("LAY_TD105_FAIL_STEP") == step:
                    raise SystemExit(18)
                """,
            )
            write_executable(
                scripts / "check-architecture.sh",
                """
                #!/usr/bin/env python3
                import os
                from pathlib import Path

                with Path(os.environ["LAY_TD105_LOG"]).open("a", encoding="utf-8") as handle:
                    handle.write("check\\n")
                if os.environ.get("LAY_TD105_FAIL_STEP") == "check":
                    raise SystemExit(19)
                """,
            )

            environment = os.environ.copy()
            environment["PATH"] = f"{root / 'bin'}:{environment['PATH']}"
            environment["LAY_TD105_LOG"] = str(log)
            if fail_step is not None:
                environment["LAY_TD105_FAIL_STEP"] = fail_step
            result = subprocess.run(
                [str(wrapper)],
                cwd=root,
                env=environment,
                check=False,
                capture_output=True,
                text=True,
                timeout=10,
            )
            steps_run = log.read_text(encoding="utf-8").splitlines()
            return result.returncode, steps_run

    def test_success_runs_each_step_once_in_order(self) -> None:
        returncode, steps = self.run_wrapper()

        self.assertEqual(0, returncode)
        self.assertEqual(
            ["graphify", "prune", "prune-check", "binding", "receipt", "check"],
            steps,
        )

    def test_each_failure_stops_before_later_steps(self) -> None:
        expected = {
            "graphify": ["graphify"],
            "prune": ["graphify", "prune"],
            "prune-check": ["graphify", "prune", "prune-check"],
            "binding": ["graphify", "prune", "prune-check", "binding"],
            "receipt": [
                "graphify",
                "prune",
                "prune-check",
                "binding",
                "receipt",
            ],
            "check": [
                "graphify",
                "prune",
                "prune-check",
                "binding",
                "receipt",
                "check",
            ],
        }

        for fail_step, expected_steps in expected.items():
            with self.subTest(fail_step=fail_step):
                returncode, steps = self.run_wrapper(fail_step)
                self.assertNotEqual(0, returncode)
                self.assertEqual(expected_steps, steps)


if __name__ == "__main__":
    unittest.main()
