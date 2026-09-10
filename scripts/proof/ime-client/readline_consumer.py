#!/usr/bin/python3
"""Apply an emitted terminal frame to a real GNU Readline buffer."""

from __future__ import annotations

import hashlib, json, os
from pathlib import Path
import pty, select, subprocess, sys, tempfile, time


SCHEMA = "lay.readline-consumer.v1"
PROMPT = b"TD125_READLINE_READY> "
MAX_INPUT_BYTES = 16 * 1024
IO_TIMEOUT_S = 3.0
INPUTRC = 'set editing-mode emacs\n"\\C-?": backward-delete-char\n'
BASH_PROGRAM = 'IFS= read -r -e -p "$1" line; status=$?; printf "%s\\0" "$line"; exit "$status"'


class ConsumerError(RuntimeError):
    pass


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def bounded_json_input() -> dict[str, str]:
    raw = sys.stdin.buffer.read(MAX_INPUT_BYTES + 1)
    if len(raw) > MAX_INPUT_BYTES:
        raise ConsumerError("input exceeds 16384 bytes")
    try:
        value = json.loads(raw.decode("utf-8"))
    except (UnicodeError, json.JSONDecodeError) as error:
        raise ConsumerError(f"invalid UTF-8 JSON input: {error}") from error
    if not isinstance(value, dict) or set(value) != {"initial", "payload", "marker"}:
        raise ConsumerError("input must have exactly initial, payload, marker")
    if not all(isinstance(value[name], str) for name in value):
        raise ConsumerError("initial, payload and marker must be strings")
    for name in ("initial", "payload", "marker"):
        if "\0" in value[name] or "\n" in value[name] or "\r" in value[name]:
            raise ConsumerError(f"{name} contains a line terminator")
    if len(value["marker"]) != 1 or ord(value["marker"]) < 0x20 or value["marker"] == "\x7f":
        raise ConsumerError("marker must be one printable Unicode scalar")
    return value


def read_through(fd: int, marker: bytes, timeout: float) -> bytes:
    deadline = time.monotonic() + timeout
    collected = bytearray()
    while marker not in collected:
        remaining = deadline - time.monotonic()
        if remaining <= 0 or not select.select([fd], [], [], remaining)[0]:
            raise ConsumerError(f"timed out waiting for {marker!r}")
        block = os.read(fd, 4096)
        if not block:
            raise ConsumerError(f"EOF before {marker!r}")
        collected.extend(block)
    return bytes(collected)


def write_all(fd: int, data: bytes, timeout: float) -> None:
    deadline = time.monotonic() + timeout
    offset = 0
    while offset < len(data):
        remaining = deadline - time.monotonic()
        if remaining <= 0 or not select.select([], [fd], [], remaining)[1]:
            raise ConsumerError("timed out writing PTY input")
        offset += os.write(fd, data[offset:])


def wait_process(process: subprocess.Popen[bytes], timeout: float) -> int:
    if process.poll() is not None:
        return int(process.returncode)
    pidfd = os.pidfd_open(process.pid)
    try:
        if not select.select([pidfd], [], [], timeout)[0]:
            raise ConsumerError("timed out waiting for Bash")
    finally:
        os.close(pidfd)
    return process.wait()


def stop_process(process: subprocess.Popen[bytes]) -> None:
    if process.poll() is not None:
        return
    process.terminate()
    try:
        wait_process(process, 1.0)
    except ConsumerError:
        process.kill()
        wait_process(process, 1.0)


def consume(value: dict[str, str]) -> dict[str, object]:
    bash = Path("/bin/bash").resolve(strict=True)
    master, slave = pty.openpty()
    process: subprocess.Popen[bytes] | None = None
    try:
        with tempfile.TemporaryDirectory(prefix="lay-readline-consumer-") as temporary:
            inputrc = Path(temporary) / "inputrc"
            inputrc.write_text(INPUTRC, encoding="utf-8")
            environment = os.environ.copy()
            environment.pop("BASH_ENV", None)
            environment.pop("ENV", None)
            environment.update({"HOME": temporary, "INPUTRC": str(inputrc),
                                "LC_ALL": "C.UTF-8", "LANG": "C.UTF-8", "TERM": "dumb"})
            process = subprocess.Popen(
                [str(bash), "--noprofile", "--norc", "-c", BASH_PROGRAM,
                 "lay-readline-consumer", PROMPT.decode("ascii")],
                stdin=slave, stdout=subprocess.PIPE, stderr=slave,
                env=environment, close_fds=True, start_new_session=True,
            )
            os.close(slave)
            slave = -1
            prompt_bytes = read_through(master, PROMPT, IO_TIMEOUT_S)
            process_exe = Path(f"/proc/{process.pid}/exe").resolve(strict=True)
            if process_exe != bash:
                raise ConsumerError(
                    f"read -e process is not pinned Bash: {process_exe} != {bash}"
                )
            written = (value["initial"] + value["payload"] + value["marker"] + "\n").encode()
            write_all(master, written, IO_TIMEOUT_S)
            assert process.stdout is not None
            framed = read_through(process.stdout.fileno(), b"\0", IO_TIMEOUT_S)
            status = wait_process(process, IO_TIMEOUT_S)
            if status != 0 or framed.count(b"\0") != 1 or not framed.endswith(b"\0"):
                raise ConsumerError(f"Bash result framing failed: status={status}")
            final_line = framed[:-1].decode("utf-8")
            return {"schema": SCHEMA, "status": "PASS",
                    "scope": "emitter_payload_consumed_by_gnu_readline",
                    "line": final_line,
                    "written_sha256": hashlib.sha256(written).hexdigest(),
                    "prompt_sha256": hashlib.sha256(prompt_bytes).hexdigest(),
                    "identity": {"bash_path": str(bash), "bash_sha256": sha256(bash),
                                 "process_exe_path": str(process_exe),
                                 "implementation": "bash_read_e_embedded_readline",
                                 "inputrc_sha256": sha256(inputrc), "locale": "C.UTF-8"}}
    finally:
        if process is not None:
            stop_process(process)
        os.close(master)
        if slave >= 0:
            os.close(slave)


def main() -> int:
    try:
        result = consume(bounded_json_input())
    except Exception as error:
        result = {"schema": SCHEMA, "status": "ERROR",
                  "error": f"{type(error).__name__}: {error}"}
        print(json.dumps(result, ensure_ascii=False, sort_keys=True),
              file=sys.stderr, flush=True)
        return 1
    print(json.dumps(result, ensure_ascii=False, sort_keys=True), flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
