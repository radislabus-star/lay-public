#!/usr/bin/env python3
"""Crash-safe content-addressed storage for sealed research payloads."""

from __future__ import annotations

import argparse
import csv
import fcntl
import hashlib
import json
import os
import re
import shutil
import stat
import sys
import tempfile
import secrets
from contextlib import contextmanager
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from pathlib import Path, PurePosixPath
from typing import Iterator, Sequence


DEFAULT_INVENTORY = "tech_debt/evidence/td103-payload-inventory-v1.tsv"
DEFAULT_PARENT_INVENTORY = "tech_debt/evidence/td103-parent-mode-inventory-v1.tsv"
DEFAULT_STORE = "/home/ubu/projects/lay-immutable-evidence/content-addressed-v1"
HASH_RE = re.compile(r"[0-9a-f]{64}")
OUTPUT_RE = re.compile(r"td103-[a-z0-9-]+-v[0-9]+\.json")
COPY_CHUNK_BYTES = 8 * 1024 * 1024


class EvidenceError(RuntimeError):
    pass


class InjectedFailure(EvidenceError):
    pass


@dataclass(frozen=True)
class Entry:
    path: str
    size_bytes: int
    mode: int
    sha256: str
    git_state: str
    selection: str
    independent_backup_count_before: int

    def catalog_record(self) -> dict[str, object]:
        record = asdict(self)
        record["mode"] = f"{self.mode:04o}"
        return record


@dataclass(frozen=True)
class ParentContract:
    path: str
    mode: int
    uid: int
    gid: int

    def catalog_record(self) -> dict[str, object]:
        record = asdict(self)
        record["mode"] = f"{self.mode:04o}"
        return record


@dataclass
class FaultInjector:
    point: str | None = None
    fired: bool = False

    def hit(self, point: str) -> None:
        if self.point == point and not self.fired:
            self.fired = True
            raise InjectedFailure(f"injected failure at {point}")


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        while chunk := handle.read(COPY_CHUNK_BYTES):
            digest.update(chunk)
    return digest.hexdigest()


def fsync_directory(path: Path) -> None:
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def atomic_json(path: Path, payload: object, fault: FaultInjector | None = None) -> None:
    encoded = (json.dumps(payload, indent=2, sort_keys=True) + "\n").encode()
    if path.is_symlink():
        raise EvidenceError(f"refusing symlink output: {path}")
    if path.exists():
        if not path.is_file() or path.read_bytes() != encoded:
            raise EvidenceError(f"refusing to replace existing output: {path}")
        return
    if not path.parent.is_dir() or path.parent.is_symlink():
        raise EvidenceError(f"output parent must be an existing regular directory: {path.parent}")
    descriptor, temporary_name = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    temporary = Path(temporary_name)
    try:
        with os.fdopen(descriptor, "wb") as handle:
            handle.write(encoded)
            handle.flush()
            os.fchmod(handle.fileno(), 0o444)
            os.fsync(handle.fileno())
        if fault is not None:
            fault.hit("before-catalog-publish")
        try:
            os.link(temporary, path, follow_symlinks=False)
        except FileExistsError:
            if path.is_symlink() or not path.is_file() or path.read_bytes() != encoded:
                raise EvidenceError(f"refusing to replace concurrently published output: {path}")
        else:
            fsync_directory(path.parent)
    finally:
        temporary.unlink(missing_ok=True)
        fsync_directory(path.parent)


def validate_output_path(repo: Path, output: Path) -> Path:
    candidate = output if output.is_absolute() else repo / output
    candidate = candidate.absolute()
    allowed_parent = repo / "tech_debt" / "evidence"
    if candidate.parent != allowed_parent or OUTPUT_RE.fullmatch(candidate.name) is None:
        raise EvidenceError(
            "--output must be a direct tech_debt/evidence/td103-*-vN.json path"
        )
    if allowed_parent.is_symlink() or allowed_parent.resolve(strict=True) != allowed_parent:
        raise EvidenceError("TD-103 output directory must not contain symlink indirection")
    return candidate


def reject_symlink_components(path: Path) -> None:
    absolute = path.absolute()
    current = Path(absolute.anchor)
    for part in absolute.parts[1:]:
        current /= part
        if current.is_symlink():
            raise EvidenceError(f"symlink component is forbidden: {current}")
        if not current.exists():
            break


def validate_relative_path(raw: str) -> str:
    candidate = PurePosixPath(raw)
    if candidate.is_absolute() or not candidate.parts or ".." in candidate.parts:
        raise EvidenceError(f"inventory path escapes repository: {raw!r}")
    if str(candidate) != raw or raw.startswith("./"):
        raise EvidenceError(f"inventory path is not canonical: {raw!r}")
    return raw


def load_inventory(path: Path) -> list[Entry]:
    with path.open(newline="", encoding="utf-8") as handle:
        rows = list(csv.DictReader(handle, delimiter="\t"))
    if not rows:
        raise EvidenceError("inventory is empty")

    entries: list[Entry] = []
    seen_paths: set[str] = set()
    object_contracts: dict[str, tuple[int, int]] = {}
    for row in rows:
        relative = validate_relative_path(row["path"])
        if relative in seen_paths:
            raise EvidenceError(f"duplicate inventory path: {relative}")
        seen_paths.add(relative)
        digest = row["sha256"]
        if HASH_RE.fullmatch(digest) is None:
            raise EvidenceError(f"invalid SHA-256 for {relative}")
        try:
            size_bytes = int(row["size_bytes"])
            mode = int(row["mode"], 8)
            backup_count = int(row["independent_backup_count_before"])
        except ValueError as error:
            raise EvidenceError(f"invalid numeric field for {relative}") from error
        if size_bytes < 0 or backup_count < 0 or mode & ~0o777:
            raise EvidenceError(f"invalid size, mode, or backup count for {relative}")
        contract = (size_bytes, mode)
        previous = object_contracts.setdefault(digest, contract)
        if previous != contract:
            raise EvidenceError(f"same bytes have incompatible size/mode contracts: {digest}")
        entries.append(
            Entry(
                path=relative,
                size_bytes=size_bytes,
                mode=mode,
                sha256=digest,
                git_state=row["git_state"],
                selection=row["selection"],
                independent_backup_count_before=backup_count,
            )
        )
    return sorted(entries, key=lambda entry: entry.path)


def load_parent_inventory(path: Path) -> list[ParentContract]:
    with path.open(newline="", encoding="utf-8") as handle:
        rows = list(csv.DictReader(handle, delimiter="\t"))
    if not rows:
        raise EvidenceError("parent-mode inventory is empty")
    contracts: list[ParentContract] = []
    seen: set[str] = set()
    for row in rows:
        relative = validate_relative_path(row["path"])
        if relative in seen:
            raise EvidenceError(f"duplicate parent-mode path: {relative}")
        seen.add(relative)
        try:
            mode = int(row["mode"], 8)
            uid = int(row["uid"])
            gid = int(row["gid"])
        except ValueError as error:
            raise EvidenceError(f"invalid parent-mode field for {relative}") from error
        if mode & ~0o777 or uid < 0 or gid < 0:
            raise EvidenceError(f"invalid parent mode or owner for {relative}")
        contracts.append(ParentContract(relative, mode, uid, gid))
    return sorted(contracts, key=lambda contract: contract.path)


class EvidenceStore:
    def __init__(
        self,
        repo: Path,
        store: Path,
        entries: Sequence[Entry],
        parent_contracts: Sequence[ParentContract],
        fault: FaultInjector | None = None,
    ) -> None:
        self.repo = repo.resolve()
        requested_store = store.absolute()
        reject_symlink_components(requested_store)
        self.store = requested_store
        self.entries = list(entries)
        self.by_path = {entry.path: entry for entry in entries}
        self.parent_contracts = {contract.path: contract for contract in parent_contracts}
        self.fault = fault or FaultInjector()
        if self.store == self.repo or self.repo in self.store.parents:
            raise EvidenceError("content store must be outside the repository")
        required_parents = {str(PurePosixPath(entry.path).parent) for entry in entries}
        missing_parents = sorted(required_parents - self.parent_contracts.keys())
        if missing_parents:
            raise EvidenceError(f"parent-mode inventory is incomplete: {', '.join(missing_parents)}")

    @property
    def journal_path(self) -> Path:
        return self.store / "transactions" / "td103-v1.jsonl"

    def source_path(self, entry: Entry) -> Path:
        path = self.repo.joinpath(*PurePosixPath(entry.path).parts)
        if self.repo not in path.parents:
            raise EvidenceError(f"source path escaped repository: {entry.path}")
        return path

    def object_path(self, entry: Entry) -> Path:
        return self.store / "objects" / "sha256" / entry.sha256[:2] / entry.sha256

    def ensure_store_directory(self, relative: str = ".") -> Path:
        candidate = PurePosixPath(relative)
        if candidate.is_absolute() or ".." in candidate.parts:
            raise EvidenceError(f"store directory escapes root: {relative}")
        target = self.store.joinpath(*candidate.parts)
        current = Path(self.store.anchor)
        for part in self.store.parts[1:]:
            current /= part
            if current.is_symlink():
                raise EvidenceError(f"symlink component is forbidden: {current}")
            if not current.exists():
                current.mkdir()
                fsync_directory(current.parent)
            elif not current.is_dir():
                raise EvidenceError(f"store component is not a directory: {current}")
        for part in candidate.parts:
            if part == ".":
                continue
            current /= part
            if current.is_symlink():
                raise EvidenceError(f"symlink component is forbidden: {current}")
            if not current.exists():
                current.mkdir()
                fsync_directory(current.parent)
            elif not current.is_dir():
                raise EvidenceError(f"store component is not a directory: {current}")
        if current != target:
            raise EvidenceError(f"store directory resolution mismatch: {target}")
        return target

    def parent_path(self, contract: ParentContract) -> Path:
        path = self.repo.joinpath(*PurePosixPath(contract.path).parts)
        if self.repo not in path.parents:
            raise EvidenceError(f"parent path escaped repository: {contract.path}")
        return path

    @contextmanager
    def lock(self, exclusive: bool = True) -> Iterator[None]:
        self.ensure_store_directory()
        lock_path = self.store / "LOCK"
        existed = os.path.lexists(lock_path)
        if existed:
            metadata = lock_path.lstat()
            if not stat.S_ISREG(metadata.st_mode):
                raise EvidenceError(f"store lock is not a regular file: {lock_path}")
        descriptor = os.open(
            lock_path,
            os.O_CREAT | os.O_RDWR | getattr(os, "O_NOFOLLOW", 0),
            0o600,
        )
        try:
            if not stat.S_ISREG(os.fstat(descriptor).st_mode):
                raise EvidenceError(f"store lock is not a regular file: {lock_path}")
            if not existed:
                fsync_directory(self.store)
            fcntl.flock(descriptor, fcntl.LOCK_EX if exclusive else fcntl.LOCK_SH)
            yield
        finally:
            fcntl.flock(descriptor, fcntl.LOCK_UN)
            os.close(descriptor)

    def journal(self, event: str, entry: Entry | None = None, **facts: object) -> None:
        journal_parent = self.ensure_store_directory("transactions")
        if os.path.lexists(self.journal_path):
            metadata = self.journal_path.lstat()
            if not stat.S_ISREG(metadata.st_mode):
                raise EvidenceError(f"durable journal is not a regular file: {self.journal_path}")
        record: dict[str, object] = {
            "event": event,
            "pid": os.getpid(),
            "time_utc": datetime.now(timezone.utc).isoformat(),
        }
        if entry is not None:
            record.update({"path": entry.path, "sha256": entry.sha256})
        record.update(facts)
        descriptor = os.open(
            self.journal_path,
            os.O_CREAT | os.O_APPEND | os.O_WRONLY | getattr(os, "O_NOFOLLOW", 0),
            0o600,
        )
        with os.fdopen(descriptor, "a", encoding="utf-8") as handle:
            handle.write(json.dumps(record, sort_keys=True) + "\n")
            handle.flush()
            os.fsync(handle.fileno())
        fsync_directory(journal_parent)

    def journal_records(self) -> list[dict[str, object]]:
        if not self.journal_path.exists():
            return []
        if self.journal_path.is_symlink():
            raise EvidenceError(f"durable journal must not be a symlink: {self.journal_path}")
        records, valid_bytes = self.parse_journal_bytes(self.journal_path.read_bytes(), False)
        if valid_bytes != self.journal_path.stat().st_size:
            raise EvidenceError("durable journal contains an unrepaired torn tail")
        return records

    @staticmethod
    def parse_journal_bytes(
        payload: bytes, allow_torn_tail: bool
    ) -> tuple[list[dict[str, object]], int]:
        records: list[dict[str, object]] = []
        lines = payload.splitlines(keepends=True)
        valid_bytes = 0
        for index, encoded_line in enumerate(lines):
            is_final = index == len(lines) - 1
            if not encoded_line.endswith(b"\n"):
                if allow_torn_tail and is_final:
                    return records, valid_bytes
                raise EvidenceError(f"unterminated durable journal record at line {index + 1}")
            try:
                record = json.loads(encoded_line)
            except (UnicodeDecodeError, json.JSONDecodeError):
                if allow_torn_tail and is_final:
                    return records, valid_bytes
                raise EvidenceError(f"invalid durable journal record at line {index + 1}")
            if not isinstance(record, dict):
                raise EvidenceError(f"invalid durable journal object at line {index + 1}")
            records.append(record)
            valid_bytes += len(encoded_line)
        return records, valid_bytes

    def repair_journal_tail(self) -> int:
        if not self.journal_path.exists():
            return 0
        if self.journal_path.is_symlink():
            raise EvidenceError(f"durable journal must not be a symlink: {self.journal_path}")
        descriptor = os.open(
            self.journal_path,
            os.O_RDWR | getattr(os, "O_NOFOLLOW", 0),
        )
        with os.fdopen(descriptor, "r+b") as handle:
            payload = handle.read()
            _, valid_bytes = self.parse_journal_bytes(payload, True)
            removed = len(payload) - valid_bytes
            if removed:
                handle.truncate(valid_bytes)
                handle.flush()
                os.fsync(handle.fileno())
        if removed:
            fsync_directory(self.journal_path.parent)
            self.journal("journal-tail-truncated", removed_bytes=removed)
        return removed

    def verify_parent_contract(self, contract: ParentContract, require_mode: bool = True) -> None:
        path = self.parent_path(contract)
        reject_symlink_components(path)
        if path.is_symlink() or self.repo not in path.resolve(strict=True).parents:
            raise EvidenceError(f"payload parent escapes repository: {path}")
        metadata = path.stat()
        if not stat.S_ISDIR(metadata.st_mode):
            raise EvidenceError(f"payload parent is not a directory: {path}")
        if (metadata.st_uid, metadata.st_gid) != (contract.uid, contract.gid):
            raise EvidenceError(f"payload parent ownership drift: {path}")
        actual_mode = stat.S_IMODE(metadata.st_mode)
        if require_mode and actual_mode != contract.mode:
            raise EvidenceError(f"payload parent mode drift: {path}: {actual_mode:04o}")

    def verify_source_parent(self, entry: Entry, require_mode: bool = True) -> None:
        relative = str(PurePosixPath(entry.path).parent)
        contract = self.parent_contracts.get(relative)
        if contract is None:
            raise EvidenceError(f"payload parent not inventoried: {relative}")
        self.verify_parent_contract(contract, require_mode=require_mode)

    def recover_parent_modes(self) -> int:
        pending: dict[str, str] = {}
        for record in self.journal_records():
            parent = record.get("parent")
            if not isinstance(parent, str):
                continue
            if record.get("event") == "parent-write-intent":
                pending[parent] = str(record.get("original_mode"))
            elif record.get("event") in {"parent-mode-restored", "parent-mode-recovered"}:
                pending.pop(parent, None)
        recovered = 0
        for parent, original_mode in sorted(pending.items()):
            contract = self.parent_contracts.get(parent)
            if contract is None or original_mode != f"{contract.mode:04o}":
                raise EvidenceError(f"journal parent-mode contract drift: {parent}")
            path = self.parent_path(contract)
            self.verify_parent_contract(contract, require_mode=False)
            os.chmod(path, contract.mode)
            fsync_directory(path)
            self.journal("parent-mode-recovered", parent=parent, original_mode=original_mode)
            recovered += 1
        for contract in self.parent_contracts.values():
            self.verify_parent_contract(contract)
        return recovered

    @contextmanager
    def writable_parent(self, parent: Path) -> Iterator[None]:
        relative = parent.relative_to(self.repo).as_posix()
        contract = self.parent_contracts.get(relative)
        if contract is None:
            raise EvidenceError(f"payload parent not inventoried: {relative}")
        self.verify_parent_contract(contract)
        if contract.mode & stat.S_IWUSR:
            yield
            return
        original_mode = f"{contract.mode:04o}"
        self.journal("parent-write-intent", parent=relative, original_mode=original_mode)
        os.chmod(parent, contract.mode | stat.S_IWUSR)
        fsync_directory(parent)
        try:
            self.fault.hit("after-parent-chmod")
            yield
        finally:
            os.chmod(parent, contract.mode)
            fsync_directory(parent)
            self.journal("parent-mode-restored", parent=relative, original_mode=original_mode)

    @staticmethod
    def verify_file(path: Path, entry: Entry, require_mode: bool = True) -> None:
        try:
            metadata = path.stat()
        except FileNotFoundError as error:
            raise EvidenceError(f"missing payload: {path}") from error
        if not stat.S_ISREG(metadata.st_mode):
            raise EvidenceError(f"payload target is not a regular file: {path}")
        actual_mode = stat.S_IMODE(metadata.st_mode)
        if metadata.st_size != entry.size_bytes:
            raise EvidenceError(f"size mismatch for {path}: {metadata.st_size}")
        if require_mode and actual_mode != entry.mode:
            raise EvidenceError(f"mode mismatch for {path}: {actual_mode:04o}")
        actual_hash = sha256_file(path)
        if actual_hash != entry.sha256:
            raise EvidenceError(f"SHA-256 mismatch for {path}: {actual_hash}")

    def verify_object(self, entry: Entry) -> None:
        path = self.object_path(entry)
        if path.is_symlink():
            raise EvidenceError(f"content object must not be a symlink: {path}")
        reject_symlink_components(path.parent)
        resolved_parent = path.parent.resolve(strict=True)
        if self.store not in resolved_parent.parents:
            raise EvidenceError(f"content object directory escapes store: {path.parent}")
        self.verify_file(path, entry)

    def verify_projection(self, entry: Entry, require_parent_mode: bool = True) -> str:
        self.verify_source_parent(entry, require_mode=require_parent_mode)
        source = self.source_path(entry)
        self.verify_file(source, entry)
        if not source.is_symlink():
            return "regular"
        raw_target = os.readlink(source)
        if os.path.isabs(raw_target):
            raise EvidenceError(f"projection target must be relative: {source}")
        if source.resolve(strict=True) != self.object_path(entry).resolve(strict=True):
            raise EvidenceError(f"projection points to wrong object: {source}")
        return "symlink"

    def ensure_object(self, entry: Entry) -> bool:
        self.verify_source_parent(entry)
        source = self.source_path(entry)
        destination = self.object_path(entry)
        if destination.is_symlink():
            raise EvidenceError(f"content object must not be a symlink: {destination}")
        if destination.exists():
            self.verify_object(entry)
            return False
        if source.is_symlink():
            raise EvidenceError(f"cannot stage an object from a source symlink: {source}")
        self.verify_file(source, entry)
        self.ensure_store_directory(f"objects/sha256/{entry.sha256[:2]}")
        descriptor, temporary_name = tempfile.mkstemp(
            prefix=f".{entry.sha256}.", dir=destination.parent
        )
        temporary = Path(temporary_name)
        try:
            with source.open("rb") as input_handle, os.fdopen(descriptor, "wb") as output_handle:
                shutil.copyfileobj(input_handle, output_handle, COPY_CHUNK_BYTES)
                output_handle.flush()
                os.fchmod(output_handle.fileno(), entry.mode)
                os.fsync(output_handle.fileno())
            self.verify_file(temporary, entry)
            self.fault.hit("before-object-publish")
            os.replace(temporary, destination)
            fsync_directory(destination.parent)
            self.verify_object(entry)
            self.journal("object-published", entry, object=str(destination))
            return True
        finally:
            temporary.unlink(missing_ok=True)

    def stage(self, selected: Sequence[Entry]) -> dict[str, object]:
        self.ensure_store_directory()
        unique = {entry.sha256: entry for entry in selected}
        missing_bytes = sum(
            entry.size_bytes
            for entry in unique.values()
            if not self.object_path(entry).exists()
        )
        free_bytes = shutil.disk_usage(self.store).free
        if free_bytes < missing_bytes:
            raise EvidenceError(
                f"insufficient store capacity: need {missing_bytes} bytes, have {free_bytes}"
            )
        created = 0
        for entry in selected:
            created += int(self.ensure_object(entry))
        return {
            "entries_checked": len(selected),
            "free_bytes_before": free_bytes,
            "missing_object_bytes_before": missing_bytes,
            "objects_created": created,
        }

    def backup_state(self, entry: Entry) -> str | None:
        state: str | None = None
        for record in self.journal_records():
            if record.get("path") != entry.path or record.get("sha256") != entry.sha256:
                continue
            event = record.get("event")
            if event == "source-rename-intent":
                state = "intent"
            elif event == "source-renamed-to-backup":
                state = "renamed"
            elif event == "backup-remove-intent":
                state = "removing"
            elif event in {
                "backup-removed-during-recovery",
                "projection-committed",
                "projection-reconstructed-from-object",
            }:
                state = None
        return state

    def finish_source_backup(
        self, entry: Entry, source: Path, backup: Path, backup_state: str | None
    ) -> None:
        if backup_state == "renamed":
            raise EvidenceError(f"journal says source is already backed up: {source}")
        if not os.path.lexists(backup):
            if backup_state != "intent":
                self.journal("source-rename-intent", entry, backup=backup.name)
            try:
                os.link(source, backup, follow_symlinks=False)
            except FileExistsError as error:
                raise EvidenceError(f"transaction backup appeared concurrently: {backup}") from error
            fsync_directory(source.parent)
        if backup.is_symlink():
            raise EvidenceError(f"transaction backup must not be a symlink: {backup}")
        self.verify_file(source, entry)
        self.verify_file(backup, entry)
        if not os.path.samefile(source, backup):
            raise EvidenceError(f"transaction backup is not the admitted source inode: {backup}")
        source.unlink()
        fsync_directory(source.parent)
        self.journal("source-renamed-to-backup", entry)
        self.fault.hit("after-backup-rename")

    def publish_projection_link(self, entry: Entry, source: Path, destination: Path) -> None:
        relative_target = os.path.relpath(destination, source.parent)
        token = f"{os.getpid()}.{secrets.token_hex(8)}"
        temporary = source.with_name(f".{source.name}.td103-link.{token}")
        self.journal(
            "temporary-link-intent",
            entry,
            target=relative_target,
            temporary=temporary.name,
        )
        try:
            os.symlink(relative_target, temporary)
            os.replace(temporary, source)
            fsync_directory(source.parent)
            self.journal("projection-published", entry, target=relative_target)
        finally:
            if os.path.lexists(temporary):
                if not temporary.is_symlink() or os.readlink(temporary) != relative_target:
                    raise EvidenceError(f"temporary projection identity drift: {temporary}")
                temporary.unlink()
                fsync_directory(source.parent)

    def externalize_entry(self, entry: Entry) -> str:
        source = self.source_path(entry)
        destination = self.object_path(entry)
        backup = source.with_name(f".{source.name}.td103-backup")
        self.verify_object(entry)

        with self.writable_parent(source.parent):
            backup_state = self.backup_state(entry)
            backup_present = os.path.lexists(backup)
            if backup_present and backup.is_symlink():
                raise EvidenceError(f"transaction backup must not be a symlink: {backup}")
            if source.is_symlink():
                self.verify_projection(entry, require_parent_mode=False)
                if backup_present:
                    if backup_state not in {"intent", "renamed", "removing"}:
                        raise EvidenceError(f"transaction backup lacks journal provenance: {backup}")
                    self.verify_file(backup, entry)
                    if backup_state != "removing":
                        self.journal("backup-remove-intent", entry)
                    backup.unlink()
                    fsync_directory(source.parent)
                    self.fault.hit("after-backup-unlink")
                    self.journal("projection-committed", entry)
                elif backup_state in {"intent", "renamed", "removing"}:
                    self.journal("projection-committed", entry)
                return "already-externalized"

            if source.exists() and backup_present:
                if backup_state != "intent":
                    raise EvidenceError(f"ambiguous source and transaction backup coexist: {source}")
                self.finish_source_backup(entry, source, backup, backup_state)
            if source.exists():
                self.finish_source_backup(entry, source, backup, backup_state)
            elif backup_present:
                if backup_state not in {"intent", "renamed", "removing"}:
                    raise EvidenceError(f"transaction backup lacks journal provenance: {backup}")
            else:
                self.publish_projection_link(entry, source, destination)
                self.verify_projection(entry, require_parent_mode=False)
                self.journal("projection-reconstructed-from-object", entry)
                return "reconstructed"

            self.verify_file(backup, entry)
            self.publish_projection_link(entry, source, destination)
            self.fault.hit("after-link-publish")
            self.verify_projection(entry, require_parent_mode=False)
            self.journal("backup-remove-intent", entry)
            backup.unlink()
            fsync_directory(source.parent)
            self.fault.hit("after-backup-unlink")
            self.journal("projection-committed", entry)
            return "externalized"

    def externalize(self, selected: Sequence[Entry]) -> dict[str, object]:
        self.stage(selected)
        changed = 0
        for entry in selected:
            changed += int(self.externalize_entry(entry) in {"externalized", "reconstructed"})
        return {"entries_externalized": changed, "entries_checked": len(selected)}

    def materialize_entry(self, entry: Entry) -> str:
        self.verify_source_parent(entry)
        source = self.source_path(entry)
        destination = self.object_path(entry)
        self.verify_object(entry)
        if source.exists() and not source.is_symlink():
            self.verify_file(source, entry)
            return "already-materialized"
        if not source.is_symlink():
            raise EvidenceError(f"projection is neither regular nor symlink: {source}")
        self.verify_projection(entry)
        with self.writable_parent(source.parent):
            descriptor, temporary_name = tempfile.mkstemp(
                prefix=f".{source.name}.", dir=source.parent
            )
            temporary = Path(temporary_name)
            try:
                with destination.open("rb") as input_handle, os.fdopen(descriptor, "wb") as output_handle:
                    shutil.copyfileobj(input_handle, output_handle, COPY_CHUNK_BYTES)
                    output_handle.flush()
                    os.fchmod(output_handle.fileno(), entry.mode)
                    os.fsync(output_handle.fileno())
                self.verify_file(temporary, entry)
                os.replace(temporary, source)
                fsync_directory(source.parent)
                self.verify_file(source, entry)
                self.journal("projection-materialized", entry)
                return "materialized"
            finally:
                temporary.unlink(missing_ok=True)

    def materialize(self, selected: Sequence[Entry]) -> dict[str, object]:
        changed = 0
        for entry in selected:
            changed += int(self.materialize_entry(entry) == "materialized")
        return {"entries_materialized": changed, "entries_checked": len(selected)}

    def verify(self, selected: Sequence[Entry], include_objects: bool = True) -> dict[str, object]:
        objects: set[str] = set()
        projections = {"regular": 0, "symlink": 0}
        logical_bytes = 0
        for entry in selected:
            source = self.source_path(entry)
            projections[self.verify_projection(entry)] += 1
            logical_bytes += entry.size_bytes
            if include_objects:
                self.verify_object(entry)
                objects.add(entry.sha256)
        return {
            "entries_verified": len(selected),
            "logical_bytes_verified": logical_bytes,
            "objects_verified": len(objects) if include_objects else 0,
            "projections": projections,
            "verdict": "PASS",
        }

    def expected_object_directories(self) -> list[Path]:
        prefixes = sorted({entry.sha256[:2] for entry in self.entries})
        return [
            self.store / "objects",
            self.store / "objects" / "sha256",
            *(self.store / "objects" / "sha256" / prefix for prefix in prefixes),
        ]

    def validate_object_tree_membership(self) -> None:
        expected_files = {self.object_path(entry) for entry in self.entries}
        expected_directories = set(self.expected_object_directories())
        objects_root = self.store / "objects"
        actual_files: set[Path] = set()
        actual_directories: set[Path] = set()
        for directory, child_directories, filenames in os.walk(objects_root, followlinks=False):
            current = Path(directory)
            actual_directories.add(current)
            for child in child_directories:
                path = current / child
                if path.is_symlink():
                    raise EvidenceError(f"symlink directory in object tree: {path}")
            for filename in filenames:
                path = current / filename
                if path.is_symlink() or not path.is_file():
                    raise EvidenceError(f"non-regular content object: {path}")
                actual_files.add(path)
        extras = sorted(str(path) for path in actual_files - expected_files)
        missing = sorted(str(path) for path in expected_files - actual_files)
        extra_directories = sorted(str(path) for path in actual_directories - expected_directories)
        missing_directories = sorted(str(path) for path in expected_directories - actual_directories)
        if extras or missing or extra_directories or missing_directories:
            raise EvidenceError(
                "object tree membership drift: "
                f"extra_files={extras}, missing_files={missing}, "
                f"extra_directories={extra_directories}, missing_directories={missing_directories}"
            )

    def object_tree_sealed(self) -> bool:
        try:
            self.validate_object_tree_membership()
            return all(
                stat.S_IMODE(directory.stat().st_mode) == 0o555
                and not directory.is_symlink()
                for directory in self.expected_object_directories()
            )
        except (EvidenceError, FileNotFoundError):
            return False

    def seal_objects(self) -> dict[str, object]:
        self.verify(self.entries)
        self.validate_object_tree_membership()
        directories = self.expected_object_directories()
        for directory in sorted(directories, key=lambda path: len(path.parts), reverse=True):
            os.chmod(directory, 0o555)
            fsync_directory(directory)
        if not self.object_tree_sealed():
            raise EvidenceError("object tree did not reach sealed 0555 state")
        return {
            "directories_sealed": len(directories),
            "objects_verified": len({entry.sha256 for entry in self.entries}),
            "object_tree_mode": "0555",
            "verdict": "PASS",
        }

    def catalog(self) -> dict[str, object]:
        unique: dict[str, Entry] = {}
        for entry in self.entries:
            unique.setdefault(entry.sha256, entry)
        backup_counts = {entry.independent_backup_count_before for entry in self.entries}
        if len(backup_counts) != 1:
            raise EvidenceError("inventory has inconsistent independent backup counts")
        backup_count = backup_counts.pop()
        return {
            "schema": "lay.research-evidence-catalog.v1",
            "store_root": str(self.store),
            "same_filesystem_ownership_move": self.repo.stat().st_dev == self.store.stat().st_dev,
            "independent_backup_count_before": backup_count,
            "independent_backup_count_after": backup_count,
            "object_tree_sealed": self.object_tree_sealed(),
            "entries": [entry.catalog_record() for entry in self.entries],
            "parent_modes": [
                contract.catalog_record()
                for contract in sorted(self.parent_contracts.values(), key=lambda item: item.path)
            ],
            "summary": {
                "logical_bytes": sum(entry.size_bytes for entry in self.entries),
                "paths": len(self.entries),
                "unique_object_bytes": sum(entry.size_bytes for entry in unique.values()),
                "unique_objects": len(unique),
            },
        }


def select_entries(entries: Sequence[Entry], paths: Sequence[str], all_entries: bool) -> list[Entry]:
    if all_entries == bool(paths):
        raise EvidenceError("select exactly one of --all or one or more --path")
    if all_entries:
        return list(entries)
    by_path = {entry.path: entry for entry in entries}
    unknown = sorted(set(paths) - by_path.keys())
    if unknown:
        raise EvidenceError(f"path not present in inventory: {', '.join(unknown)}")
    return [by_path[path] for path in paths]


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--inventory", type=Path, default=Path(DEFAULT_INVENTORY))
    parser.add_argument(
        "--parent-inventory", type=Path, default=Path(DEFAULT_PARENT_INVENTORY)
    )
    parser.add_argument("--store", type=Path, default=Path(DEFAULT_STORE))
    parser.add_argument(
        "--fault",
        choices=[
            "before-object-publish",
            "after-parent-chmod",
            "after-backup-rename",
            "after-link-publish",
            "after-backup-unlink",
            "before-catalog-publish",
        ],
    )
    subparsers = parser.add_subparsers(dest="action", required=True)
    for action in ("stage", "externalize", "materialize", "verify"):
        command = subparsers.add_parser(action)
        command.add_argument("--all", action="store_true")
        command.add_argument("--path", action="append", default=[])
        command.add_argument("--output", type=Path)
    catalog = subparsers.add_parser("catalog")
    catalog.add_argument("--output", type=Path, required=True)
    seal = subparsers.add_parser("seal")
    seal.add_argument("--output", type=Path, required=True)
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    repo = args.repo.absolute()
    inventory = args.inventory if args.inventory.is_absolute() else repo / args.inventory
    parent_inventory = (
        args.parent_inventory
        if args.parent_inventory.is_absolute()
        else repo / args.parent_inventory
    )
    entries = load_inventory(inventory)
    parent_contracts = load_parent_inventory(parent_inventory)
    store = EvidenceStore(
        repo, args.store, entries, parent_contracts, FaultInjector(args.fault)
    )
    output = getattr(args, "output", None)
    try:
        if output is not None:
            output = validate_output_path(store.repo, output)
        if args.action == "catalog":
            with store.lock():
                store.repair_journal_tail()
                store.recover_parent_modes()
                result = store.catalog()
                atomic_json(output, result, store.fault)
        elif args.action == "seal":
            with store.lock():
                store.repair_journal_tail()
                store.recover_parent_modes()
                result = store.seal_objects()
                result.update({"action": args.action, "store_root": str(store.store)})
                atomic_json(output, result)
        else:
            selected = select_entries(entries, args.path, args.all)
            with store.lock():
                store.repair_journal_tail()
                store.recover_parent_modes()
                if args.action == "stage":
                    result = store.stage(selected)
                elif args.action == "externalize":
                    result = store.externalize(selected)
                elif args.action == "materialize":
                    result = store.materialize(selected)
                else:
                    result = store.verify(selected)
                result.update({"action": args.action, "store_root": str(store.store)})
                if output is not None:
                    atomic_json(output, result)
        if output is None:
            print(json.dumps(result, indent=2, sort_keys=True))
        return 0
    except (EvidenceError, OSError) as error:
        print(f"BLOCKED: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
