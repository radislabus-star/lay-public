"""Pair sanitized D-Bus observations and export an offline Perfetto timeline."""
from collections import Counter, defaultdict
import json
import math
from pathlib import Path
import statistics

from .receipts import MAX_BYTES, require

MEMBERS = {"ManualToggleV3", "NativeReplayProcessedV1", "VisibleTailV4",
           "CancelExactManualToggleSuppressionV2", "CancelExactManualToggleHandoffV2"}


def load_events(path):
    path = Path(path)
    require(path.stat().st_size <= MAX_BYTES, "wire capture exceeds 32 MiB")
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def analyze(events):
    require(isinstance(events, list) and events, "empty wire capture")
    pending, paired, gaps = {}, [], []
    last_stamp = -1
    for index, event in enumerate(events, 1):
        require(isinstance(event, dict), "invalid wire record")
        member, phase, stamp = (event.get(k) for k in ("member", "phase", "at_unix_us"))
        require(member in MEMBERS and phase in {"call", "return", "error"}, "unsupported wire event")
        require(type(stamp) is int and stamp >= last_stamp, "wire clock moved backwards or timestamp is invalid")
        last_stamp = stamp
        sender = event.get("sender" if phase == "call" else "destination")
        serial = event.get("serial" if phase == "call" else "reply_serial")
        require(isinstance(sender, str) and sender.startswith(":"), "invalid D-Bus sender")
        require(type(serial) is int and serial > 0, "invalid D-Bus serial")
        key = sender, serial
        if phase == "call":
            require(key not in pending, "duplicate outstanding D-Bus serial")
            pending[key] = (index, event)
            continue
        entry = pending.pop(key, None)
        if entry is None:
            gaps.append(dict(kind="ORPHAN_RETURN", line=index, member=member))
            continue
        call_line, call = entry
        require(call["member"] == member, "reply member does not match call")
        duration = stamp - call["at_unix_us"]
        require("latency_us" not in event or (type(event["latency_us"]) is int and event["latency_us"] == duration),
                "reported latency does not match timestamps")
        paired.append(dict(member=member, start=call["at_unix_us"], duration=duration,
                           call_line=call_line, return_line=index, call=call, reply=event))
    gaps.extend(dict(kind="UNPAIRED_CALL", line=line, member=event["member"]) for line, event in pending.values())
    by_member = defaultdict(list)
    for pair in paired:
        by_member[pair["member"]].append(pair["duration"])
    timing = {}
    for member, durations in sorted(by_member.items()):
        values = sorted(durations)
        timing[member] = dict(calls=len(values), median_us=statistics.median(values),
                              p95_us=values[math.ceil(len(values) * .95) - 1], max_us=max(values))
    sources, whitespace = Counter(), Counter()
    same_visible = 0
    previous_visible = None
    false_ack = 0
    for pair in paired:
        reply, call = pair["reply"], pair["call"]
        if pair["member"] == "NativeReplayProcessedV1" and reply.get("value") is False and call.get("epoch", 0):
            false_ack += 1
        if pair["member"] == "VisibleTailV4" and reply["phase"] == "return":
            source = reply.get("source")
            # Only known observation labels may enter the public summary.
            sources[source if source in {"passive:committed-tail", "committed-tail", "active-preedit", "unknown"} else "other"] += 1
            tail = reply.get("trailing_whitespace", [])
            require(isinstance(tail, list) and all(type(x) is int and x in {9, 10, 13, 32, 160} for x in tail),
                    "unexpected trailing-whitespace metadata")
            whitespace[",".join(map(str, tail)) or "none"] += 1
            identity = tuple(reply.get(k) for k in ("source", "epoch", "path", "text_sha256", "layout_is_ru"))
            if reply.get("text_sha256") and identity == previous_visible:
                same_visible += 1
            previous_visible = identity
    errors = [p for p in paired if p["reply"]["phase"] == "error"]
    summary = dict(schema="lay.native-wire-diagnostic.v1", records=len(events), paired_calls=len(paired),
        pairing="COMPLETE" if not gaps else "INCOMPLETE", gaps=gaps, rpc_timing=timing,
        rpc_errors=len(errors), observed_false_ack_excluding_empty_epoch_probe=false_ack,
        consecutive_unchanged_visible_reads=same_visible, visible_sources=dict(sources),
        trailing_whitespace_codepoints=dict(whitespace),
        first_rpc_error=dict(member=errors[0]["member"], call_line=errors[0]["call_line"],
                             return_line=errors[0]["return_line"]) if errors else None,
        first_failed_transition="UNKNOWN_WITHOUT_OPERATION_EXPECTATIONS",
        note="RPC duration is not key-to-visible latency. False ACKs and repeated reads alone do not establish a defect.")
    trace = [{"ph": "M", "pid": 1, "tid": 0, "name": "process_name", "args": {"name": "Lay recorded D-Bus"}}]
    lanes, lane_ids = defaultdict(list), {}
    origin = min(e["at_unix_us"] for e in events)
    for pair in sorted(paired, key=lambda p: p["start"]):
        member, start = pair["member"], pair["start"]
        available = lanes[member]
        lane = next((i for i, end in enumerate(available) if end <= start), len(available))
        if lane == len(available):
            available.append(0)
            lane_ids[member, lane] = len(lane_ids) + 1
            trace.append(dict(ph="M", pid=1, tid=lane_ids[member, lane], name="thread_name",
                              args=dict(name=member + (" #" + str(lane + 1) if lane else ""))))
        available[lane] = start + pair["duration"]
        reply = pair["reply"]
        args = {"call_line": pair["call_line"], "return_line": pair["return_line"], "rpc_error": reply["phase"] == "error"}
        # No paths, text, hashes of user text, errors or arbitrary input strings.
        for key in ("epoch", "text_chars", "status"):
            if type(reply.get(key)) is int:
                args[key] = reply[key]
        for key in ("value", "layout_is_ru", "field_present"):
            if type(reply.get(key)) is bool:
                args[key] = reply[key]
        trace.append(dict(ph="X", pid=1, tid=lane_ids[member, lane], name=member, cat="lay.dbus",
                          ts=start - origin, dur=pair["duration"], args=args))
    return summary, {"traceEvents": trace, "displayTimeUnit": "ms"}


def compare(before, after):
    members = sorted(before["rpc_timing"].keys() | after["rpc_timing"].keys())
    return dict(pairing_complete=before["pairing"] == after["pairing"] == "COMPLETE",
        calls={m: dict(before=before["rpc_timing"].get(m, {}).get("calls", 0),
                       after=after["rpc_timing"].get(m, {}).get("calls", 0)) for m in members},
        runtime_and_actor_identity="NOT_PRESENT_IN_WIRE_EVENTS_CHECK_MATRIX_OR_CAPTURE_MANIFEST",
        causal_attribution="NOT_ESTABLISHED_BY_RPC_COUNTS")
