"""Configuration rejection must happen before SSH or any proof staging."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
HELPERS = ("run-atomic-full-route-remote.sh", "stage-atomic-proof-engine-remote.sh")


class ProofRemoteConfigTests(unittest.TestCase):
    def test_shell_syntax(self):
        for name in (*HELPERS, "remote-config.sh"):
            subprocess.run(["bash", "-n", str(ROOT / "scripts/proof" / name)], check=True)
        subprocess.run(["sh", "-n", str(ROOT / "scripts/proof/remote-config.sh")], check=True)

    def test_invalid_configuration_never_contacts_worker(self):
        for name in HELPERS:
            for key, value in (("LAY_PROOF_REMOTE", None),
                               ("LAY_PROOF_REMOTE", "-oProxyCommand=command"),
                               ("LAY_PROOF_REMOTE", "worker;command"),
                               ("LAY_PROOF_ROOT", None),
                               ("LAY_PROOF_ROOT", "/workspace/../private"),
                               ("LAY_PROOF_ROOT", "/workspace/proof'"),
                               ("LAY_PROOF_ROOT", "/"),
                               ("LAY_PROOF_ROOT", "relative/proof")):
                with self.subTest(helper=name, key=key, value=value), tempfile.TemporaryDirectory() as tmp:
                    marker = Path(tmp) / "contacted"
                    fake = Path(tmp) / "ssh"
                    fake.write_text('#!/bin/sh\ntouch "$SSH_TEST_MARKER"\nexit 99\n')
                    fake.chmod(0o755)
                    env = {k: v for k, v in os.environ.items() if not k.startswith("LAY_PROOF_")}
                    env.update(PATH=tmp + os.pathsep + env["PATH"], SSH_TEST_MARKER=str(marker),
                               LAY_PROOF_REMOTE="builder@worker.example")
                    for path_key in ("ROOT", "MUTTER_ROOT", "SHELL_ROOT", "IBUS_BUILD",
                                     "MODELS_ROOT", "TARGET_DIR", "SOURCE_PARENT"):
                        env["LAY_PROOF_" + path_key] = "/workspace/worker/proof"
                    if value is None:
                        env.pop(key, None)
                    else:
                        env[key] = value
                    result = subprocess.run(["bash", str(ROOT / "scripts/proof" / name)],
                                            env=env, capture_output=True, timeout=5)
                    self.assertNotEqual(result.returncode, 0)
                    self.assertFalse(marker.exists(), result.stderr.decode())

    def test_explicit_alias_and_paths_are_admitted(self):
        source = ROOT / "scripts/proof/remote-config.sh"
        subprocess.run(["sh", "-c", '. "$1"; lay_proof_validate_host builder@worker.example; '
                        'lay_proof_validate_path LAY_PROOF_ROOT /workspace/worker/proof',
                        "proof-config-test", str(source)], check=True)


if __name__ == "__main__":
    unittest.main()
