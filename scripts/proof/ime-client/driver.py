#!/usr/bin/python3
"""Versioned V2 private IBus.InputContext -> actual Lay factory proof."""
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import time
import traceback

import gi
gi.require_version('IBus', '1.0')
from gi.repository import Gio, GLib, IBus

ROOT = Path('/tmp/proof')
DEPLOYED = Path('/tmp/deployed/bundle')
LOADER = DEPLOYED / 'lib/ld-linux-x86-64.so.2'
LIBS = DEPLOYED / 'lib'
DAEMON_BIN = DEPLOYED / 'bin/ibus-daemon'
CANDIDATE = Path(os.environ['IME_CLIENT_CANDIDATE'])
SYSTEM_DAEMON = Path('/usr/bin/ibus-daemon')
SEED_BIN = Path('/usr/libexec/ibus-engine-simple')
IBUS_NAME = 'org.freedesktop.IBus'
IBUS_PATH = '/org/freedesktop/IBus'
IBUS_ENGINE_NAME = 'org.freedesktop.IBus.Lay'
BRIDGE = 'io.github.radislabus_star.LayIme'
BRIDGE_PATH = '/io/github/radislabus_star/LayIme'
SEED_ENGINE = 'xkb:us::eng'
US_ENGINE = 'lay-ime-us'
RU_ENGINE = 'lay-ime-ru'
RELEASE_MASK = 1 << 30
MARKER_PATH = '/io/github/radislabus_star/LayIme/ContextAdmission'
MARKER_INTERFACE = 'io.github.radislabus_star.LayIme.ContextAdmission'
barrier_events = []
barrier_waiter = None
STARTUP_SCHEDULE = os.environ.get('IME_CLIENT_STARTUP_SCHEDULE', 'immediate')
SCENARIO_SET = os.environ.get('IME_CLIENT_SCENARIO_SET', 'restoration')
STARTUP_PROOF_PROFILE = os.environ.get('IME_CLIENT_STARTUP_PROOF_PROFILE', 'legacy')
COMPLETED_STATUS = ('COMPLETED_3_LIFECYCLE_CASES' if SCENARIO_SET == 'lifecycle'
                    else 'COMPLETED_4_TERMINAL_DELIVERY_CASES' if SCENARIO_SET == 'terminal-delivery'
                    else 'COMPLETED_3_MANUAL_CASES' if SCENARIO_SET == 'manual-toggle'
                    else 'COMPLETED_2_FIRST_WORD_CASES' if SCENARIO_SET == 'first-word'
                    else 'COMPLETED_1_FIRST_WORD_CASE' if SCENARIO_SET in ('first-word-us', 'first-word-ru')
                    else 'COMPLETED_1_STARTUP_CASE' if SCENARIO_SET in ('fresh-preedit', 'startup-only', 'packages-absent-literal')
                    else 'COMPLETED_5_CASES_AUTHORITY_ON')
startup_observed = False
startup_measurement = None
KEYCODES = {' ': 57, 'l': 38, 'j': 36, 'v': 47, 'о': 36, 'м': 47, 'д': 38}
KEYCODES.update({'a': 30, 'g': 34, 'h': 35, 'd': 32, 'п': 34, 'р': 35, 'в': 32})
KEYCODES.update({'е': 20, 'к': 19, 'а': 33, 'и': 48, 'с': 46, 'т': 49, 'ь': 50})

receipt = {
    'proof_contract': 'lay.ime-client.actual-input-context.v2',
    'predecessor_contract': {
        'version': 'v1',
        'git_commit': '708245298a3f553ac3c52243728c02ba6344a140',
        'git_blob': '04f7dbac56c92dbe0238c0754bcb6db4e242256c',
        'sha256': '9ece223f6689323e5cae3fc5f27cf990d37b86dff5e9f6e0ff3212d4dd488750',
        'normalized_sha256': '669e3ef88cc2639794fde79dcb9132b99ebcaf142cafe39065376f42c4a056dc',
        'comparison': 'IMMUTABLE_HISTORY_NOT_BASELINE_PARITY',
    },
    'scope': 'private actual IBus.InputContext plus exact candidate Lay factory',
    'runtime_authority_changed': False,
    'configuration': 'private_authority_on_correction_enabled',
    'startup_schedule': STARTUP_SCHEDULE,
    'scenario_set': SCENARIO_SET,
    'startup_proof_profile': STARTUP_PROOF_PROFILE,
    'unknown_authority_verdict': 'REQUIRES_SAME_RUN_POSITIVE_CONTROL',
    'gui': 'NOT_TESTED',
    'physical_keyboard': 'NOT_TESTED',
    'general_wave_quality': 'NOT_TESTED',
    'diagnostic_controls': {
        'private_debug_action_log': True,
        'production_config_changed': False,
        'zero_key_readiness_control': False,
        'literal_space_is_observed_rearm_boundary': True,
        'post_verdict_trace_drain_ms': 1250,
        'trace_drain_can_change_behavior_verdict': False,
        'absent_trace_does_not_prove_missing_callback': True,
        'surrounding_text_advertised_and_published': True,
        'native_unhandled_fixture_input_applied_by_client': True,
    },
    'cases': [],
}
started = time.monotonic()
daemon = None
daemon_log = None
ibus_bus = None
session_connection = None
contexts = []
streams = {}
case_name = 'bootstrap'


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def exact_file_from_env(path_name, hash_name):
    path = Path(os.environ[path_name])
    expected = os.environ[hash_name]
    assert len(expected) == 64 and all(c in '0123456789abcdef' for c in expected)
    assert path.is_file(), path
    actual = sha256(path)
    assert actual == expected, (path_name, expected, actual)
    return {'path': str(path), 'sha256': actual, 'bytes': path.stat().st_size}


def startup_trace_measurement(path):
    if path.stat().st_size > 1024 * 1024:
        raise ValueError('startup trace exceeds bounded proof input')
    rows = []
    for line in path.read_text(encoding='utf-8').splitlines():
        rows.append(json.loads(line))
    completed = [(index, row) for index, row in enumerate(rows)
                 if row.get('kind') == 'ibus_startup_warmup' and row.get('stage') == 'completed']
    factories = [(index, row) for index, row in enumerate(rows)
                 if row.get('kind') == 'ibus_context_admission'
                 and row.get('member') == 'CreateEngine'
                 and row.get('stage') == 'factory_acquisition_state']
    exact = [row for row in rows if row.get('kind') == 'ibus_exact_authority_warmup'
             and row.get('stage') == 'completed']
    assert len(completed) == 1 and factories and len(exact) == 1
    assert completed[0][0] < factories[0][0]
    expected_exact = STARTUP_PROOF_PROFILE != 'absent'
    assert completed[0][1].get('exact_available') is expected_exact
    assert exact[0].get('available') is expected_exact
    assert completed[0][1].get('exact_available') is exact[0].get('available')
    assert completed[0][1].get('l2_complete') is True
    expected_l2 = STARTUP_PROOF_PROFILE != 'absent'
    assert completed[0][1].get('l2_available') is expected_l2
    assert completed[0][1].get('l2_candidate_ready') is expected_l2
    return {'warmup_join_us': completed[0][1]['elapsed_us'],
            'exact_warmup_us': exact[0]['elapsed_us'],
            'exact_available': exact[0]['available'],
            'l2_complete': completed[0][1]['l2_complete'],
            'l2_available': completed[0][1]['l2_available'],
            'l2_candidate_ready': completed[0][1]['l2_candidate_ready'],
            'warmup_event_index': completed[0][0],
            'first_factory_event_index': factories[0][0],
            'factory_after_warmup': True,
            'production_percentile_claim': False}


def open_stream(name):
    path = ROOT / name
    streams[name] = path.open('x', encoding='utf-8', buffering=1)
    return streams[name]


input_log = open_stream('actual-input.jsonl')
output_log = open_stream('commit-output.jsonl')
internal_log = open_stream('internal-observations.jsonl')
client_log = open_stream('client-projection.jsonl')


def emit(handle, **row):
    row = {'case': case_name, 'monotonic_ns': time.monotonic_ns(), **row}
    handle.write(json.dumps(row, ensure_ascii=False) + '\n')
    return row


def drain():
    context = GLib.MainContext.default()
    while context.pending():
        context.iteration(False)


def wait_until(predicate, label, timeout=8.0):
    deadline = time.monotonic() + timeout
    last = None
    while time.monotonic() < deadline:
        drain()
        try:
            last = predicate()
            if last:
                return last
        except Exception as error:
            last = f'{type(error).__name__}: {error}'
        time.sleep(0.01)
    raise TimeoutError(f'{label}: last={last!r}')


def rpc(connection, destination, path, interface, method, signature=None, values=()):
    parameters = GLib.Variant(signature, values) if signature else None
    return connection.call_sync(
        destination, path, interface, method, parameters, None,
        Gio.DBusCallFlags.NO_AUTO_START, 2500, None).unpack()


def name_has_owner(connection, name):
    return rpc(connection, 'org.freedesktop.DBus', '/org/freedesktop/DBus',
               'org.freedesktop.DBus', 'NameHasOwner', '(s)', (name,))[0]


def name_owner(connection, name):
    return rpc(connection, 'org.freedesktop.DBus', '/org/freedesktop/DBus',
               'org.freedesktop.DBus', 'GetNameOwner', '(s)', (name,))[0]


def current_context_path():
    connection = ibus_bus.get_connection()
    return str(rpc(connection, IBUS_NAME, IBUS_PATH,
                   'org.freedesktop.DBus.Properties', 'Get', '(ss)',
                   (IBUS_NAME, 'CurrentInputContext'))[0])


def current_engine_name():
    desc = ibus_bus.get_global_engine()
    return desc.get_name() if desc is not None else None


def bridge_snapshot(label):
    values = rpc(session_connection, BRIDGE, BRIDGE_PATH, BRIDGE,
                 'VisibleTailV3')
    state, text, layout_is_ru, epoch, path, focus_receipt = values
    input_state = rpc(session_connection, BRIDGE, BRIDGE_PATH, BRIDGE,
                      'InputState')[0]
    row = emit(internal_log, kind='bridge_snapshot', label=label,
               canonical_context=current_context_path(), state=state, text=text,
               layout_is_ru=layout_is_ru, epoch=epoch, engine_path=path,
               focus_receipt=focus_receipt, input_state=input_state)
    return row


def subscribe_barriers(connection):
    def received(_connection, sender, _path, _interface, _member, parameters, _data):
        if parameters.get_type_string() != '(t)' or len(barrier_events) >= 128:
            return
        nonce = int(parameters.unpack()[0])
        if nonce <= 0:
            return
        barrier_events.append(emit(internal_log, kind='fixture_barrier_observed',
                                   sender=sender, nonce=nonce))
        if barrier_waiter is not None:
            barrier_waiter.quit()

    subscription = connection.signal_subscribe(
        None, MARKER_INTERFACE, 'Barrier', MARKER_PATH, None,
        Gio.DBusSignalFlags.NO_MATCH_RULE, received, None)
    rule = ("type='signal',interface='" + MARKER_INTERFACE +
            "',member='Barrier',path='" + MARKER_PATH + "'")
    rpc(connection, 'org.freedesktop.DBus', '/org/freedesktop/DBus',
        'org.freedesktop.DBus', 'AddMatch', '(s)', (rule,))
    emit(internal_log, kind='fixture_subscription_acknowledged',
         subscription=int(subscription))


def barrier_checkpoint():
    # A same-connection reply plus dispatch drains older marker notifications.
    current_context_path()
    drain()
    return len(barrier_events)


def await_barrier(checkpoint, sender):
    global barrier_waiter

    def matching():
        return [event for event in barrier_events[checkpoint:]
                if event['sender'] == sender]

    drain()
    events = matching()
    if not events:
        loop = GLib.MainLoop()
        expired = []

        def timeout():
            expired.append(True)
            loop.quit()
            return False

        timeout_id = GLib.timeout_add(2500, timeout)
        barrier_waiter = loop
        try:
            loop.run()
        finally:
            barrier_waiter = None
            if not expired:
                GLib.source_remove(timeout_id)
        events = matching()
    assert events, ('missing candidate Barrier; no retry', sender, checkpoint)
    return events[-1]


def observe_startup_schedule():
    global startup_observed
    if startup_observed or STARTUP_SCHEDULE == 'immediate':
        return
    assert STARTUP_SCHEDULE == 'post-exact-ready', STARTUP_SCHEDULE
    trace_path = ROOT / 'ibus-engine-trace.jsonl'
    loop = GLib.MainLoop()
    observed, errors = [], []
    start_ns = time.monotonic_ns()

    def inspect(*_args):
        try:
            if not trace_path.exists():
                return
            if trace_path.stat().st_size > 1024 * 1024:
                raise ValueError('startup trace exceeds bounded proof input')
            content = trace_path.read_text(encoding='utf-8')
            for line in content.splitlines(keepends=True):
                if not line.endswith('\n'):
                    continue
                row = json.loads(line)
                if (row.get('kind') == 'ibus_exact_authority_warmup'
                        and row.get('stage') == 'completed'):
                    if row.get('available') is not True:
                        raise ValueError('candidate exact authority unavailable')
                    observed.append(row)
                    loop.quit()
                    return
        except Exception as error:
            errors.append(error)
            loop.quit()

    # Subscribe before inspection so file creation/flush cannot be lost. This
    # observes batched diagnostics, never calls warmup or polls runtime state.
    monitor = Gio.File.new_for_path(str(trace_path)).monitor_file(
        Gio.FileMonitorFlags.NONE, None)
    monitor.connect('changed', inspect)
    timeout_id = GLib.timeout_add(2500, lambda: (loop.quit(), False)[1])
    try:
        inspect()
        if not observed and not errors:
            loop.run()
        if errors:
            raise errors[0]
        assert observed, 'exact startup completion not observed; no input/retry'
        startup_observed = True
        receipt['startup_observation'] = emit(
            internal_log, kind='post_exact_ready_startup_observed',
            schedule='PROCESS_COLD_INPUT_POST_EXACT_READY',
            observation_elapsed_us=(time.monotonic_ns() - start_ns) // 1000,
            candidate_warmup_us=observed[0]['elapsed_us'],
            trace_flush_delay_included=True, production_latency_proof=False)
    finally:
        monitor.cancel()
        if GLib.MainContext.default().find_source_by_id(timeout_id):
            GLib.source_remove(timeout_id)


def setup_ready(client, profile, checkpoint, expected_prefix=None, expected_manual_suffix=None):
    global startup_measurement
    connection = ibus_bus.get_connection()
    owner = name_owner(connection, IBUS_ENGINE_NAME)
    marker = await_barrier(checkpoint, owner)
    assert current_context_path() == client.path
    assert current_engine_name() == profile
    visible, output_count = client.visible, len(client.output)
    # One observe-only modifier pair, never a second Space or text mutation.
    for phase, state in (('press', 0), ('release', RELEASE_MASK)):
        handled = bool(client.context.process_key_event(0xffe1, 42, state))
        emit(input_log, kind='fixture_shift', context=client.path,
             keyval=0xffe1, keycode=42, phase=phase, state=state, handled=handled)
        assert not handled, phase
    drain()
    assert client.visible == visible and len(client.output) == output_count
    assert current_context_path() == client.path
    assert current_engine_name() == profile
    assert name_owner(connection, IBUS_ENGINE_NAME) == owner
    state = rpc(session_connection, BRIDGE, BRIDGE_PATH, BRIDGE, 'InputState')[0]
    assert state not in ('passive:unknown-context', 'passive:no-focus'), state
    if expected_manual_suffix is not None:
        assert state == 'passive:committed-tail', state
        snapshot = rpc(session_connection, BRIDGE, BRIDGE_PATH, BRIDGE, 'VisibleTailV3')
        assert snapshot[0] == 'passive:unknown-context' and not snapshot[1], snapshot
        assert client.visible == expected_manual_suffix, client.visible
    elif expected_prefix is None:
        assert state == 'passive:daemon-word-buffer', state
    else:
        snapshot = rpc(session_connection, BRIDGE, BRIDGE_PATH, BRIDGE, 'VisibleTailV3')
        assert snapshot[1] == expected_prefix and snapshot[5], snapshot
    emit(internal_log, kind='fixture_owner_ready', context=client.path,
         profile=profile, owner=owner, marker_nonce=marker['nonce'],
         input_state=state, expected_prefix_chars=(len(expected_prefix)
                                                 if expected_prefix is not None else 0))
    if startup_measurement is not None and 'client_ready_elapsed_us' not in startup_measurement:
        startup_measurement['client_ready_elapsed_us'] = (
            time.monotonic_ns() - startup_measurement['started_monotonic_ns']) // 1000
        startup_measurement['factory_owner'] = owner
    observe_startup_schedule()


def client_snapshot(client, label):
    return emit(client_log, kind='client_projection', label=label,
                context=client.path, visible=client.visible,
                cursor=client.cursor, output_count=len(client.output))


def candidate_pids(include_status=False):
    found = []
    for entry in Path('/proc').iterdir():
        if not entry.name.isdigit():
            continue
        try:
            command = (entry / 'cmdline').read_bytes().replace(b'\0', b' ').decode()
            if command.split(' ', 1)[0] == str(CANDIDATE):
                row = {'pid': int(entry.name), 'cmdline': command.strip(),
                       'ppid': int((entry / 'stat').read_text().split()[3])}
                if include_status:
                    status = (entry / 'status').read_text(encoding='utf-8')
                    memory = {}
                    for line in status.splitlines():
                        field, separator, value = line.partition(':')
                        if separator and field in ('VmRSS', 'VmHWM'):
                            amount, unit = value.split()
                            assert unit == 'kB', (field, value)
                            memory[field + '_kib'] = int(amount)
                    assert set(memory) == {'VmRSS_kib', 'VmHWM_kib'}, memory
                    row['memory_sample'] = memory
                found.append(row)
        except (FileNotFoundError, PermissionError, ProcessLookupError, UnicodeDecodeError):
            pass
    return sorted(found, key=lambda row: row['pid'])


class Client:
    def __init__(self, name):
        self.name = name
        self.context = ibus_bus.create_input_context(name)
        self.path = self.context.get_object_path()
        self.visible = ''
        self.cursor = 0
        self.surrounding_dirty = True
        self.output = []
        self.context.connect('commit-text', self.on_commit)
        self.context.connect('delete-surrounding-text', self.on_delete)
        self.context.connect('forward-key-event', self.on_forward)
        self.context.set_capabilities(int(IBus.Capabilite.PREEDIT_TEXT |
                                          IBus.Capabilite.FOCUS |
                                          IBus.Capabilite.SURROUNDING_TEXT))
        self.context.set_content_type(int(IBus.InputPurpose.FREE_FORM), 0)
        self.context.set_cursor_location(0, 0, 11, 24)
        contexts.append(self)

    def publish_surrounding(self, reason, force=False):
        # libibus caches unsent snapshots too. Publishing before its internal
        # RequireSurroundingText flag is set can suppress the later identical
        # snapshot, so retain dirty state until the engine actually requests it.
        if not self.context.needs_surrounding_text():
            return
        if not force and not self.surrounding_dirty:
            return
        self.context.set_surrounding_text(
            IBus.Text.new_from_string(self.visible), self.cursor, self.cursor)
        self.surrounding_dirty = False
        emit(client_log, kind='SurroundingTextPublished', reason=reason,
             context=self.path, visible=self.visible, cursor=self.cursor,
             anchor=self.cursor)

    def apply_native_input(self, character, reason):
        self.visible = self.visible[:self.cursor] + character + self.visible[self.cursor:]
        self.cursor += len(character)
        self.surrounding_dirty = True
        emit(client_log, kind='NativeUnhandledInput', reason=reason,
             context=self.path, character=character, visible=self.visible,
             cursor=self.cursor)

    def on_commit(self, _context, text):
        value = text.get_text()
        self.visible = self.visible[:self.cursor] + value + self.visible[self.cursor:]
        self.cursor += len(value)
        self.surrounding_dirty = True
        row = emit(output_log, kind='CommitText', context=self.path, text=value,
                   visible=self.visible, cursor=self.cursor)
        self.output.append(row)

    def on_delete(self, _context, offset, count):
        begin = max(0, min(len(self.visible), self.cursor + offset))
        end = max(begin, min(len(self.visible), begin + count))
        deleted = self.visible[begin:end]
        self.visible = self.visible[:begin] + self.visible[end:]
        self.cursor = begin
        self.surrounding_dirty = True
        row = emit(output_log, kind='DeleteSurroundingText', context=self.path,
                   offset=offset, count=count, deleted=deleted,
                   visible=self.visible, cursor=self.cursor)
        self.output.append(row)

    def on_forward(self, _context, keyval, keycode, state):
        row = emit(output_log, kind='ForwardKeyEvent', context=self.path,
                   keyval=keyval, keycode=keycode, state=state)
        self.output.append(row)

    def focus_in(self):
        self.context.focus_in()
        drain()
        self.publish_surrounding('FocusInDispatch', force=True)
        drain()

    def focus_out(self):
        self.context.focus_out()
        drain()

    def key(self, character):
        keyval = IBus.unicode_to_keyval(character)
        keycode = KEYCODES[character]
        press = self.context.process_key_event(keyval, keycode, 0)
        emit(input_log, kind='ProcessKeyEvent', context=self.path,
             character=character, keyval=keyval, keycode=keycode,
             state=0, phase='press', handled=bool(press))
        if not press:
            self.apply_native_input(character, 'ProcessKeyEvent')
        self.publish_surrounding('ProcessKeyEventPress')
        release = self.context.process_key_event(keyval, keycode, RELEASE_MASK)
        emit(input_log, kind='ProcessKeyEvent', context=self.path,
             character=character, keyval=keyval, keycode=keycode,
             state=RELEASE_MASK, phase='release', handled=bool(release))
        drain()
        self.publish_surrounding('ProcessKeyEventDispatch')
        drain()
        return bool(press)


def select_engine(name):
    global startup_measurement
    before = current_engine_name()
    before_ns = time.monotonic_ns()
    if startup_measurement is None and before == SEED_ENGINE and name in (US_ENGINE, RU_ENGINE):
        startup_measurement = {'started_monotonic_ns': before_ns, 'target_engine': name}
        receipt['startup_measurement'] = startup_measurement
    assert ibus_bus.set_global_engine(name), name
    wait_until(lambda: current_engine_name() == name,
               f'global engine did not become {name}')
    row = emit(internal_log, kind='global_engine_transition', before=before,
               after=name, elapsed_us=(time.monotonic_ns() - before_ns) // 1000)
    if startup_measurement is not None and startup_measurement.get('target_engine') == name:
        startup_measurement.setdefault('global_engine_transition_us', row['elapsed_us'])
    return row


def outputs_since(client, start):
    return client.output[start:]


def assert_no_edits(rows):
    assert not [row for row in rows if row['kind'] == 'DeleteSurroundingText'], rows


def deliver_exact_literal(client, character):
    output_start = len(client.output)
    visible_before = client.visible
    cursor_before = client.cursor
    handled = client.key(character)
    rows = outputs_since(client, output_start)
    commits = [row for row in rows if row['kind'] == 'CommitText']
    deletes = [row for row in rows if row['kind'] == 'DeleteSurroundingText']
    forwards = [row for row in rows if row['kind'] == 'ForwardKeyEvent']
    expected_visible = (visible_before[:cursor_before] + character
                        + visible_before[cursor_before:])
    assert not deletes and not forwards, rows
    assert client.visible == expected_visible, (client.visible, expected_visible, rows)
    assert client.cursor == cursor_before + len(character), (client.cursor, cursor_before, rows)
    if handled:
        assert len(commits) == 1 and commits[0]['text'] == character, rows
    else:
        assert not commits, rows
    return {'character': character, 'handled': handled, 'outputs': rows,
            'visible': client.visible}


def run_cases():
    global case_name

    case_name = 'same_context_us_to_us_authority_restoration'
    a = Client('td121-same-us')
    initial_checkpoint = barrier_checkpoint()
    a.focus_in()
    select_engine(US_ENGINE)
    wait_until(lambda: name_has_owner(session_connection, BRIDGE), 'Lay bridge owner')
    wait_until(lambda: candidate_pids(), 'actual Lay process')
    ibus_connection = ibus_bus.get_connection()
    receiver_roles = {
        'scope': 'externally_observed_bus_name_owners_not_internal_binding_export',
        'ibus_owner': name_owner(ibus_connection, IBUS_NAME),
        'engine_factory_owner': name_owner(ibus_connection, IBUS_ENGINE_NAME),
        'driver_ibus_unique_name': ibus_connection.get_unique_name(),
        'bridge_session_owner': name_owner(session_connection, BRIDGE),
        'driver_session_unique_name': session_connection.get_unique_name(),
    }
    receipt['receiver_roles'] = receiver_roles
    emit(internal_log, kind='receiver_role_snapshot', **receiver_roles)

    setup_ready(a, US_ENGINE, initial_checkpoint)

    # The literal Space is the real observed boundary that rearms KnownStart.
    # No zero-key bridge loop is allowed to stand in for this callback.
    a.key(' ')
    rearm = bridge_snapshot('known_rearm_after_literal_space')
    assert rearm['text'] == ' ', rearm
    assert rearm['focus_receipt'], rearm
    client_snapshot(a, 'after_known_rearm_boundary')
    start = len(a.output)
    a.key('l')
    before = bridge_snapshot('before_same_profile_recreation')
    canonical_before = before['canonical_context']
    handoff_checkpoint = barrier_checkpoint()
    select_engine(US_ENGINE)
    setup_ready(a, US_ENGINE, handoff_checkpoint, expected_prefix=before['text'])
    # Acquisition completes asynchronously. The next real key callback consumes
    # readiness; a bridge-only poll here would be an invalid positive control.
    a.key('j')
    a.key('v')
    after = bridge_snapshot('after_same_profile_subsequent_callbacks')
    assert after['canonical_context'] == canonical_before
    assert after['engine_path'] != before['engine_path'], (before, after)
    assert after['text'] == before['text'] + 'jv', (before, after)
    before_correction = client_snapshot(a, 'before_authority_boundary')
    a.key(' ')
    final = bridge_snapshot('after_authority_boundary')
    after_correction = client_snapshot(a, 'after_authority_boundary')
    rows = outputs_since(a, start)
    deletes = [row for row in rows if row['kind'] == 'DeleteSurroundingText']
    assert deletes, rows
    assert ''.join(row['deleted'] for row in deletes) == 'ljv', deletes
    assert before_correction['visible'].endswith('ljv'), before_correction
    assert after_correction['visible'].endswith('дом '), after_correction
    receipt['cases'].append({'case': case_name,
        'status': 'PASS_AUTHORITY_RESTORATION',
        'observed_rearm_boundary': rearm,
        'context': canonical_before, 'source_engine_path': before['engine_path'],
        'target_engine_path': after['engine_path'], 'visible': a.visible,
        'pre_correction_internal_tail': after['text'],
        'post_correction_internal_tail': final['text'],
        'expected_restoration': 'ljv -> дом', 'outputs': rows})
    a.focus_out()

    case_name = 'same_context_us_to_ru_mixed'
    b = Client('td121-mixed')
    initial_checkpoint = barrier_checkpoint()
    b.focus_in()
    select_engine(US_ENGINE)
    setup_ready(b, US_ENGINE, initial_checkpoint)
    b.key(' ')
    mixed_rearm = bridge_snapshot('mixed_known_rearm_after_literal_space')
    assert mixed_rearm['text'] == ' ', mixed_rearm
    assert mixed_rearm['focus_receipt'], mixed_rearm
    start = len(b.output)
    b.key('l')
    before = bridge_snapshot('before_us_to_ru')
    handoff_checkpoint = barrier_checkpoint()
    select_engine(RU_ENGINE)
    setup_ready(b, RU_ENGINE, handoff_checkpoint, expected_prefix=before['text'])
    b.key('о')
    b.key('м')
    final = bridge_snapshot('after_mixed_continuation')
    assert final['canonical_context'] == before['canonical_context']
    assert final['engine_path'] != before['engine_path'], (before, final)
    rows = outputs_since(b, start)
    assert final['text'] == before['text'] + 'ом', (before, final)
    assert ''.join(row.get('text', '') for row in rows if row['kind'] == 'CommitText') == 'lом'
    assert_no_edits(rows)
    mixed_client = client_snapshot(b, 'after_mixed_continuation')
    receipt['cases'].append({'case': case_name,
        'status': 'PASS_DISTINCT_MIXED_TRACKING',
        'observed_rearm_boundary': mixed_rearm,
        'context': before['canonical_context'], 'source_engine_path': before['engine_path'],
        'target_engine_path': final['engine_path'], 'visible': mixed_client['visible'],
        'internal_tail': final['text'], 'outputs': rows})
    b.focus_out()

    case_name = 'same_context_ru_to_us_mixed_retention'
    reverse = Client('td121-mixed-reverse')
    initial_checkpoint = barrier_checkpoint()
    reverse.focus_in()
    select_engine(RU_ENGINE)
    setup_ready(reverse, RU_ENGINE, initial_checkpoint)
    reverse.key(' ')
    reverse_rearm = bridge_snapshot('reverse_mixed_known_rearm_after_literal_space')
    assert reverse_rearm['text'] == ' ', reverse_rearm
    assert reverse_rearm['focus_receipt'], reverse_rearm
    start = len(reverse.output)
    reverse.key('д')
    before = bridge_snapshot('before_ru_to_us')
    handoff_checkpoint = barrier_checkpoint()
    select_engine(US_ENGINE)
    setup_ready(reverse, US_ENGINE, handoff_checkpoint, expected_prefix=before['text'])
    # The first printable callback consumes asynchronous acquisition readiness.
    reverse.key('j')
    reverse.key('v')
    final = bridge_snapshot('after_reverse_mixed_continuation')
    assert final['canonical_context'] == before['canonical_context']
    assert final['engine_path'] != before['engine_path'], (before, final)
    rows = outputs_since(reverse, start)
    assert final['text'] == before['text'] + 'jv', (before, final)
    assert ''.join(row.get('text', '') for row in rows if row['kind'] == 'CommitText') == 'дjv'
    assert_no_edits(rows)
    reverse_client = client_snapshot(reverse, 'after_reverse_mixed_continuation')
    assert reverse_client['visible'].endswith('дjv'), reverse_client
    receipt['cases'].append({'case': case_name,
        'status': 'PASS_RU_TO_US_MIXED_RETENTION',
        'observed_rearm_boundary': reverse_rearm,
        'context': before['canonical_context'], 'source_engine_path': before['engine_path'],
        'target_engine_path': final['engine_path'], 'visible': reverse_client['visible'],
        'internal_tail': final['text'], 'outputs': rows})
    reverse.focus_out()

    case_name = 'different_field_unknown_negative_authority_on'
    c = Client('td121-field-source')
    initial_checkpoint = barrier_checkpoint()
    c.focus_in()
    select_engine(US_ENGINE)
    setup_ready(c, US_ENGINE, initial_checkpoint)
    c.key(' ')
    c.key('l')
    source = bridge_snapshot('field_source_before_focus_out')
    c.focus_out()
    d = Client('td121-field-target')
    start = len(d.output)
    initial_checkpoint = barrier_checkpoint()
    d.focus_in()
    setup_ready(d, US_ENGINE, initial_checkpoint)
    # The first printable callback consumes any asynchronously completed
    # acquisition, but no observed boundary has rearmed this new field.
    d.key('l')
    d.key('j')
    d.key('v')
    target = bridge_snapshot('field_target_unknown_before_boundary')
    assert target['canonical_context'] != source['canonical_context'], (source, target)
    assert target['state'] == 'passive:unknown-context', target
    assert target['text'] == '', target
    d.key(' ')
    closed = bridge_snapshot('field_target_unknown_after_boundary')
    rows = outputs_since(d, start)
    assert ''.join(row.get('text', '') for row in rows if row['kind'] == 'CommitText') == 'ljv '
    assert_no_edits(rows)
    target_client = client_snapshot(d, 'field_target_after_unknown_boundary')
    receipt['cases'].append({'case': case_name,
        'status': 'PASS_DIFFERENT_FIELD_UNKNOWN_REFUSAL',
        'source_context': source['canonical_context'],
        'target_context': target['canonical_context'], 'source_visible': c.visible,
        'target_visible': target_client['visible'],
        'unknown_state_before_boundary': target['state'],
        'post_boundary_state': closed['state'], 'outputs': rows})
    d.focus_out()

    case_name = 'unknown_start_negative_authority_on'
    e = Client('td121-unknown')
    initial_checkpoint = barrier_checkpoint()
    e.focus_in()
    select_engine(US_ENGINE)
    setup_ready(e, US_ENGINE, initial_checkpoint)
    start = len(e.output)
    e.key('l')
    e.key('j')
    e.key('v')
    unknown = bridge_snapshot('unknown_before_boundary')
    assert unknown['state'] == 'passive:unknown-context', unknown
    assert unknown['text'] == '', unknown
    e.key(' ')
    closed = bridge_snapshot('unknown_closed_by_boundary')
    rows = outputs_since(e, start)
    assert ''.join(row.get('text', '') for row in rows if row['kind'] == 'CommitText') == 'ljv '
    assert_no_edits(rows)
    unknown_client = client_snapshot(e, 'unknown_after_boundary')
    receipt['cases'].append({'case': case_name,
        'status': 'PASS_UNKNOWN_START_REFUSAL',
        'authority_positive_case': 'same_context_us_to_us_authority_restoration',
        'context': unknown['canonical_context'], 'visible': e.visible,
        'unknown_state_before_boundary': unknown['state'],
        'post_boundary_state': closed['state'],
        'client_projection': unknown_client, 'outputs': rows})
    e.focus_out()


def run_lifecycle_cases():
    """Real IBus lifecycle dispatch; no claim about lexical restoration quality."""
    global case_name

    case_name = 'same_engine_refocus_via_dummy_context_discards_word'
    client = Client('td121-lifecycle')
    checkpoint = barrier_checkpoint()
    client.focus_in()
    select_engine(US_ENGINE)
    wait_until(lambda: name_has_owner(session_connection, BRIDGE), 'Lay bridge owner')
    setup_ready(client, US_ENGINE, checkpoint)
    client.key(' ')
    assert client.key('l')
    before = bridge_snapshot('lifecycle_before_refocus')
    visible = client.visible
    output_count = len(client.output)
    checkpoint = barrier_checkpoint()
    client.focus_out()
    client.focus_in()
    # Global IBus routes a real focus gap through its dummy context. Returning
    # to this client must not resurrect word authority from the previous field.
    setup_ready(client, US_ENGINE, checkpoint)
    after = bridge_snapshot('lifecycle_after_refocus')
    assert after['engine_path'] == before['engine_path'], (before, after)
    assert after['canonical_context'] == client.path
    assert after['state'] == 'passive:unknown-context', after
    assert after['text'] == '' and not after['focus_receipt'], after
    assert client.visible == visible and len(client.output) == output_count
    client.key(' ')
    rearmed = bridge_snapshot('lifecycle_refocus_rearmed_by_boundary')
    assert rearmed['text'] == ' ' and rearmed['focus_receipt'], rearmed
    assert client.key('j'), 'a post-boundary printable must be admitted'
    continued = bridge_snapshot('lifecycle_refocus_continuation')
    assert continued['text'] == ' j', continued
    assert client.visible == visible + ' j'
    assert_no_edits(outputs_since(client, output_count))
    receipt['cases'].append({'case': case_name, 'status': 'PASS_LIFECYCLE',
                            'before': before, 'empty': after, 'after': continued,
                            'visible': client.visible})

    # Both transitions use the real client interface while refocusing. A fresh
    # IBus may send FocusInId; controlled plain-FocusIn/Get interleavings belong
    # to the real-zbus callback regressions, not an invented daemon option.
    for transition in ('terminal_content_type', 'reset'):
        case_name = 'refocus_' + transition + '_drops_word_not_context'
        before = bridge_snapshot('lifecycle_before_word_loss')
        visible = client.visible
        output_count = len(client.output)
        checkpoint = barrier_checkpoint()
        client.focus_out()
        if transition == 'terminal_content_type':
            client.context.set_content_type(int(IBus.InputPurpose.TERMINAL), 0)
            emit(input_log, kind='SetContentType', context=client.path,
                 purpose=int(IBus.InputPurpose.TERMINAL), hints=0)
        client.focus_in()
        if transition == 'reset':
            client.context.reset()
            emit(input_log, kind='Reset', context=client.path)
        setup_ready(client, US_ENGINE, checkpoint)
        empty = bridge_snapshot('lifecycle_word_revoked_context_ready')
        assert empty['state'] == 'passive:unknown-context', empty
        assert empty['text'] == '' and not empty['focus_receipt'], empty
        assert empty['engine_path'] == before['engine_path'], (before, empty)
        assert empty['canonical_context'] == client.path
        assert client.visible == visible and len(client.output) == output_count
        # One real boundary, then useful admitted text. Never retry a failed key.
        client.key(' ')
        rearmed = bridge_snapshot('lifecycle_rearmed_after_word_loss')
        assert rearmed['text'] == ' ' and rearmed['focus_receipt'], rearmed
        assert client.key('l'), 'a post-boundary printable must be admitted'
        fresh = bridge_snapshot('lifecycle_fresh_word_after_loss')
        assert fresh['text'] == ' l', fresh
        assert client.visible == visible + ' l', client.visible
        assert_no_edits(outputs_since(client, output_count))
        receipt['cases'].append({'case': case_name, 'status': 'PASS_LIFECYCLE',
                                'before': before, 'empty': empty, 'after': fresh,
                                'visible': client.visible})
    client.focus_out()


class TerminalClient(Client):
    def __init__(self, name):
        self.consumer_errors = []
        super().__init__(name)
        self.context.set_capabilities(int(IBus.Capabilite.PREEDIT_TEXT |
                                          IBus.Capabilite.FOCUS))
        self.context.set_content_type(int(IBus.InputPurpose.TERMINAL), 0)

    def publish_surrounding(self, reason, force=False):
        pass

    def on_commit(self, _context, text):
        from readline_consumer import consume
        value = text.get_text()
        try:
            result = consume({'initial': self.visible, 'payload': value, 'marker': 'X'})
            assert result['line'].endswith('X'), result
            self.visible = result['line'][:-1]
            self.cursor = len(self.visible)
            self.output.append(emit(output_log, kind='CommitText', context=self.path,
                                    text=value, visible=self.visible, cursor=self.cursor,
                                    consumer=result))
        except Exception as error:
            self.consumer_errors.append(repr(error))


def observe_terminal_prefetch(snapshot):
    """Observe real publication for this word; never inject or retry input."""
    trace_path = ROOT / 'ibus-engine-trace.jsonl'
    loop = GLib.MainLoop()
    observed, errors = [], []
    started_ns = time.monotonic_ns()

    def inspect(*_args):
        try:
            if not trace_path.exists():
                return
            assert trace_path.stat().st_size <= 1024 * 1024, 'bounded terminal trace'
            for line in trace_path.read_text(encoding='utf-8').splitlines(keepends=True):
                if not line.endswith('\n'):
                    continue
                row = json.loads(line)
                if (row.get('kind') == 'ibus_space_prefetch_timing'
                        and row.get('outcome') == 'prepared'
                        and row.get('engine_path') == snapshot['engine_path']
                        and row.get('tail_epoch') == snapshot['epoch']):
                    observed.append(row)
                    loop.quit()
                    return
        except Exception as error:
            errors.append(error)
            loop.quit()

    monitor = Gio.File.new_for_path(str(trace_path)).monitor_file(Gio.FileMonitorFlags.NONE, None)
    monitor.connect('changed', inspect)
    timeout_id = GLib.timeout_add(2500, lambda: (loop.quit(), False)[1])
    try:
        inspect()
        if not observed and not errors:
            loop.run()
        if errors:
            raise errors[0]
        assert observed, 'native word did not publish prefetch; no input/retry'
        return emit(internal_log, kind='terminal_prefetch_publication_observed',
                    snapshot=snapshot, publication=observed[0],
                    observation_elapsed_us=(time.monotonic_ns() - started_ns) // 1000,
                    trace_flush_delay_included=True, production_latency_proof=False)
    finally:
        monitor.cancel()
        if GLib.MainContext.default().find_source_by_id(timeout_id):
            GLib.source_remove(timeout_id)


def run_terminal_delivery_cases():
    global case_name
    assert private_config['auto_replace'] and private_config['nanda_precognition']
    receipt['diagnostic_controls']['surrounding_text_advertised_and_published'] = False
    receipt['terminal_scope'] = ('real IBus native callbacks and GNU Readline replacement; '
                                 'Space after observed publication; no GNOME keyboard encoder')
    for shape, old, new in [('shortening', 'дподпись', 'подпись'),
                            ('growth', 'средсва', 'средства'),
                            ('equal', 'провреь', 'проверь')]:
        case_name = 'terminal_delivery_' + shape
        client = TerminalClient('td125-' + case_name)
        checkpoint = barrier_checkpoint()
        client.focus_in()
        select_engine(RU_ENGINE)
        setup_ready(client, RU_ENGINE, checkpoint)
        prefix = 'метка '
        for character in prefix + old:
            assert not client.key(character), 'terminal glyph must use native input'
        assert client.visible == prefix + old, client.visible
        assert not client.output, client.output
        before = bridge_snapshot('terminal_word_before_space')
        assert before['text'] == prefix + old and before['focus_receipt'], before
        prepared = observe_terminal_prefetch(before)
        assert client.key(' '), 'published correction must consume Space'
        expected = prefix + new + ' '
        assert not client.consumer_errors, client.consumer_errors
        assert client.visible == expected, (client.visible, expected)
        assert len(client.output) == 1, client.output
        edit = client.output[0]
        assert edit['kind'] == 'CommitText' and edit['text'] == '\x7f' * len(old) + new + ' ', edit
        assert edit['consumer']['line'] == expected + 'X', edit
        assert client.cursor == len(expected), client.cursor
        assert not client.key('а'), 'next native letter must follow the replacement'
        assert client.visible == expected + 'а' and client.cursor == len(expected) + 1
        assert len(client.output) == 1, 'no second text owner after replacement'
        after = bridge_snapshot('terminal_after_replacement_and_native_letter')
        assert after['text'] == client.visible, after
        receipt['cases'].append({'case': case_name, 'status': 'PASS_TERMINAL_DELIVERY',
                                'shape': shape, 'source': old, 'replacement': new,
                                'before': before, 'prepared': prepared, 'edit': edit,
                                'after': after, 'visible': client.visible, 'cursor': client.cursor})
        client.focus_out()

    case_name = 'terminal_native_hint_visible'
    client = TerminalClient('td125-native-hint')
    preedits = []
    client.context.connect('update-preedit-text', lambda _c, text, cursor, visible:
        preedits.append(emit(output_log, kind='UpdatePreeditText', text=text.get_text(),
                             cursor=cursor, visible=bool(visible))))
    checkpoint = barrier_checkpoint()
    client.focus_in()
    select_engine(RU_ENGINE)
    setup_ready(client, RU_ENGINE, checkpoint)
    for character in ' пров':
        assert not client.key(character), 'hint input stays native'
    wait_until(lambda: any(row['visible'] and row['text'] for row in preedits),
               'visible terminal suggestion after native input')
    assert client.visible == ' пров' and not client.output, client.visible
    receipt['cases'].append({'case': case_name, 'status': 'PASS_TERMINAL_NATIVE_HINT',
                            'preedits': preedits, 'visible': client.visible})
    client.focus_out()


def run_manual_toggle_cases(first_word=False):
    global case_name
    shell_requests = []
    interface = 'io.github.radislabus_star.LayDaemon'
    shell_xml = Gio.DBusNodeInfo.new_for_xml(
        '<node><interface name="' + interface + '"><method name="ActivateLayout">'
        '<arg type="s" direction="in"/><arg type="b" direction="out"/>'
        '</method></interface></node>')

    def activate_layout(_connection, sender, _path, _interface, method, params, invocation):
        assert method == 'ActivateLayout'
        layout = params.unpack()[0]
        assert layout in ('us', 'ru'), layout
        target = RU_ENGINE if layout == 'ru' else US_ENGINE
        row = emit(internal_log, kind='private_shell_activation', sender=sender,
                   target=target, production_gnome=False)
        shell_requests.append(row)
        # The real shell activation schedules IBus asynchronously. Reply before
        # its resulting factory callbacks need the source engine's write lock.
        def switched(connection, result):
            try:
                row['reply'] = connection.call_finish(result).unpack()
            except Exception as error:
                row['error'] = repr(error)
        ibus_bus.get_connection().call(
            IBUS_NAME, IBUS_PATH, IBUS_NAME, 'SetGlobalEngine',
            GLib.Variant('(s)', (target,)), None, Gio.DBusCallFlags.NO_AUTO_START,
            2500, None, switched)
        invocation.return_value(GLib.Variant('(b)', (True,)))

    registration = session_connection.register_object(
        '/io/github/radislabus_star/LayDaemon', shell_xml.interfaces[0],
        activate_layout, None, None)
    assert rpc(session_connection, 'org.freedesktop.DBus', '/org/freedesktop/DBus',
               'org.freedesktop.DBus', 'RequestName', '(su)', ('org.gnome.Shell', 0))[0] == 1
    receipt['diagnostic_controls']['surrounding_text_advertised_and_published'] = False
    receipt['manual_scope'] = 'real IBus and GNU Readline; private shell fixture; no daemon gesture'
    try:
        cases = ((False, False, False), (False, False, True)) if first_word else (
            (True, False, False), (True, True, False))
        if SCENARIO_SET in ('first-word-us', 'first-word-ru'):
            cases = tuple(case for case in cases if case[2] == (SCENARIO_SET == 'first-word-ru'))
        for leading_boundary, trailing_boundary, initial_ru in cases:
            case_name = ('first_word_' + ('ru' if initial_ru else 'us') if first_word else
                         'terminal_manual_toggle_' + ('boundary' if trailing_boundary else 'word'))
            client = TerminalClient('td121-' + case_name)
            checkpoint = barrier_checkpoint()
            client.focus_in()
            initial_engine = RU_ENGINE if initial_ru else US_ENGINE
            select_engine(initial_engine)
            wait_until(lambda: name_has_owner(session_connection, BRIDGE), 'Lay bridge owner')
            setup_ready(client, initial_engine, checkpoint)
            prefix = ' ' if leading_boundary else ''
            expected_us = prefix + ('a ' if trailing_boundary else 'ghjd')
            expected_ru = prefix + ('ф ' if trailing_boundary else 'пров')
            expected_initial = expected_ru if initial_ru else expected_us
            for character in expected_initial:
                client.key(character)
            initial = bridge_snapshot('manual_initial')
            assert client.visible == expected_initial, client.visible
            rounds = []
            for index in range(8):
                target_ru = (index % 2 == 0) != initial_ru
                target = RU_ENGINE if target_ru else US_ENGINE
                expected = expected_ru if target_ru else expected_us
                checkpoint = barrier_checkpoint()
                output_start, activation_start = len(client.output), len(shell_requests)
                reply = []
                def toggled(connection, result):
                    try:
                        reply.append(connection.call_finish(result).unpack())
                    except Exception as error:
                        reply.append(error)
                session_connection.call(
                    BRIDGE, BRIDGE_PATH, BRIDGE, 'ManualToggleV3', None, None,
                    Gio.DBusCallFlags.NO_AUTO_START, 2500, None, toggled)
                wait_until(lambda: reply, 'single ManualToggleV3 reply')
                assert reply[0] == (1, target_ru), reply
                wait_until(lambda: current_engine_name() == target, 'requested profile')
                if leading_boundary or trailing_boundary:
                    setup_ready(client, target, checkpoint, expected_prefix=expected)
                else:
                    setup_ready(client, target, checkpoint, expected_manual_suffix=expected)
                after = bridge_snapshot('manual_successor')
                assert after['canonical_context'] == initial['canonical_context']
                if leading_boundary or trailing_boundary:
                    assert after['text'] == expected and after['focus_receipt'], after
                else:
                    assert after['state'] == 'passive:unknown-context' and not after['text'], after
                assert not client.consumer_errors, client.consumer_errors
                assert client.visible == expected, (index, client.visible, expected)
                rows = outputs_since(client, output_start)
                assert len(rows) == 1 and rows[0]['kind'] == 'CommitText', rows
                assert len(shell_requests) == activation_start + 1, shell_requests
                assert not shell_requests[-1].get('error'), shell_requests[-1]
                rounds.append({'index': index, 'visible': client.visible, 'reply': reply[0],
                               'output': rows[0], 'after': after})
            receipt['cases'].append({'case': case_name, 'status': 'PASS_MANUAL_TOGGLE',
                                    'leading_boundary': leading_boundary,
                                    'initial_layout_is_ru': initial_ru, 'rounds': rounds})
            client.focus_out()

        if first_word:
            return

        case_name = 'preedit_visible_after_same_context_handoff'
        client = Client('td121-preedit')
        preedits = []
        client.context.connect('update-preedit-text', lambda _c, text, cursor, visible:
            preedits.append(emit(output_log, kind='UpdatePreeditText', text=text.get_text(),
                                 cursor=cursor, visible=bool(visible))))
        checkpoint = barrier_checkpoint()
        client.focus_in()
        select_engine(RU_ENGINE)
        setup_ready(client, RU_ENGINE, checkpoint)
        for character in ' про':
            client.key(character)
        before = bridge_snapshot('preedit_before_handoff')
        checkpoint = barrier_checkpoint()
        select_engine(RU_ENGINE)
        setup_ready(client, RU_ENGINE, checkpoint, expected_prefix=before['text'])
        start = len(preedits)
        client.key('в')
        wait_until(lambda: any(row['visible'] and row['text']
                               for row in preedits[start:]), 'visible post-handoff suggestion')
        receipt['cases'].append({'case': case_name, 'status': 'PASS_VISIBLE_PREEDIT',
                                'preedits': preedits[start:], 'visible': client.visible})
        client.focus_out()
    finally:
        session_connection.unregister_object(registration)


def run_startup_only_case():
    global case_name
    case_name = 'startup_only'
    client = Client('td121-startup-only')
    checkpoint = barrier_checkpoint()
    client.focus_in()
    select_engine(US_ENGINE)
    wait_until(lambda: name_has_owner(session_connection, BRIDGE), 'Lay bridge owner')
    setup_ready(client, US_ENGINE, checkpoint)
    receipt['cases'].append({'case': case_name, 'status': 'PASS_STARTUP_READY',
                            'visible': client.visible, 'outputs': client.output})
    client.focus_out()


def run_fresh_preedit_case():
    global case_name
    case_name = 'fresh_process_l2_preedit'
    client = Client('td121-fresh-preedit')
    preedits = []
    client.context.connect('update-preedit-text', lambda _c, text, cursor, visible:
        preedits.append(emit(output_log, kind='UpdatePreeditText', text=text.get_text(),
                             cursor=cursor, visible=bool(visible))))
    checkpoint = barrier_checkpoint()
    client.focus_in()
    select_engine(RU_ENGINE)
    wait_until(lambda: name_has_owner(session_connection, BRIDGE), 'Lay bridge owner')
    setup_ready(client, RU_ENGINE, checkpoint)
    deliveries = [deliver_exact_literal(client, character) for character in ' пров']
    wait_until(lambda: any(row['visible'] and row['text'] for row in preedits),
               'fresh-process visible L2 preedit')
    assert client.visible == ' пров', (client.visible, client.output)
    literal_control = None
    if STARTUP_PROOF_PROFILE == 'off':
        client.focus_out()
        literal = Client('td121-autocorrect-off-literal')
        checkpoint = barrier_checkpoint()
        literal.focus_in()
        select_engine(US_ENGINE)
        setup_ready(literal, US_ENGINE, checkpoint)
        deliveries_control = [deliver_exact_literal(literal, character) for character in ' ljv ']
        rows = [row for delivery in deliveries_control for row in delivery['outputs']]
        space_handled = deliveries_control[-1]['handled']
        deletes = [row for row in rows if row['kind'] == 'DeleteSurroundingText']
        assert literal.visible == ' ljv ', literal.visible
        assert not deletes, rows
        literal_control = {'input': ' ljv ', 'visible': literal.visible,
                           'outputs': rows, 'delete_count': 0,
                           'space_handled': space_handled,
                           'space_settled': literal.visible.endswith(' ')}
        literal.focus_out()
    receipt['cases'].append({'case': case_name, 'status': 'PASS_FRESH_L2_PREEDIT',
                            'preedits': preedits, 'visible': client.visible,
                            'literal_deliveries': deliveries,
                            'nanda_autocorrect': private_config['nanda_autocorrect'],
                            'literal_control': literal_control})
    if STARTUP_PROOF_PROFILE != 'off':
        client.focus_out()


def run_packages_absent_literal_case():
    global case_name
    case_name = 'packages_absent_literal'
    client = Client('td121-packages-absent')
    checkpoint = barrier_checkpoint()
    client.focus_in()
    select_engine(US_ENGINE)
    wait_until(lambda: name_has_owner(session_connection, BRIDGE), 'Lay bridge owner')
    setup_ready(client, US_ENGINE, checkpoint)
    deliveries = [deliver_exact_literal(client, character) for character in ' ljv ']
    rows = [row for delivery in deliveries for row in delivery['outputs']]
    assert client.visible == ' ljv ', client.visible
    assert_no_edits(rows)
    receipt['cases'].append({'case': case_name, 'status': 'PASS_PACKAGES_ABSENT_LITERAL',
                            'visible': client.visible, 'outputs': rows,
                            'literal_deliveries': deliveries})
    client.focus_out()


def deadline(_signum, _frame):
    raise TimeoutError('private client proof hard deadline')


def validate_private_config(config, profile):
    assert config.get('debug_action_log') is True
    assert config.get('typing_assist') is False
    if profile == 'off':
        assert config.get('nanda_autocorrect') is False
        assert config.get('auto_replace') is False
        assert config.get('auto_switch_layout') is False
        assert config.get('nanda_precognition') is True
    else:
        assert config.get('nanda_autocorrect') is True
        assert config.get('auto_replace') is True
        assert config.get('auto_switch_layout') is True
        if profile != 'legacy':
            assert config.get('nanda_precognition') is True


try:
    assert SCENARIO_SET in ('restoration', 'lifecycle', 'manual-toggle', 'terminal-delivery', 'first-word',
                            'first-word-us', 'first-word-ru', 'fresh-preedit', 'startup-only',
                            'packages-absent-literal'), SCENARIO_SET
    assert STARTUP_PROOF_PROFILE in ('legacy', 'on', 'off', 'absent'), STARTUP_PROOF_PROFILE
    signal.signal(signal.SIGALRM, deadline)
    signal.alarm(52)
    lease_id = os.environ['TD121_EXECUTION_LEASE_ID']
    assert 1 <= len(lease_id) <= 128
    assert all(c.isalnum() or c in '._:-' for c in lease_id)
    receipt['execution_lease_id'] = lease_id
    expected_hash = os.environ['EXPECTED_CANDIDATE_SHA256']
    assert len(expected_hash) == 64 and all(c in '0123456789abcdef' for c in expected_hash)
    assert CANDIDATE.is_file() and os.access(CANDIDATE, os.X_OK)
    assert sha256(CANDIDATE) == expected_hash
    private_config = json.loads((ROOT / 'config.json').read_text(encoding='utf-8'))
    validate_private_config(private_config, STARTUP_PROOF_PROFILE)
    if STARTUP_PROOF_PROFILE != 'legacy':
        receipt['configuration'] = ('private_packages_absent'
                                    if STARTUP_PROOF_PROFILE == 'absent'
                                    else 'private_autocorrect_' + STARTUP_PROOF_PROFILE + '_preedit_enabled')
    authority_files = {} if STARTUP_PROOF_PROFILE == 'absent' else {
        'l11_service': exact_file_from_env(
            'LAY_L11_SERVICE_BIN', 'EXPECTED_L11_SERVICE_SHA256'),
        'l11_package': exact_file_from_env(
            'TD121_L11_PACKAGE_PATH', 'EXPECTED_TD121_L11_PACKAGE_SHA256'),
        'l11_proof': exact_file_from_env(
            'TD121_L11_PROOF_PATH', 'EXPECTED_TD121_L11_PROOF_SHA256'),
        'l11_active_receipt': exact_file_from_env(
            'LAY_L11_RECEIPT', 'EXPECTED_LAY_L11_RECEIPT_SHA256'),
        'l2_v13': exact_file_from_env(
            'LAY_L2_PACKAGE', 'EXPECTED_LAY_L2_PACKAGE_SHA256'),
        'productive_v90_package': exact_file_from_env(
            'LAY_L2_PRODUCTIVE_V1_PACKAGE',
            'EXPECTED_LAY_L2_PRODUCTIVE_V1_PACKAGE_SHA256'),
        'productive_v90_recovery': exact_file_from_env(
            'TD121_PRODUCTIVE_V90_RECOVERY_PATH',
            'EXPECTED_TD121_PRODUCTIVE_V90_RECOVERY_SHA256'),
        'l2_v13_dafsa': exact_file_from_env(
            'LAY_L2_V13_DAFSA', 'EXPECTED_LAY_L2_V13_DAFSA_SHA256'),
    }
    if STARTUP_PROOF_PROFILE == 'absent':
        absent_names = (
            'LAY_L11_SERVICE_BIN', 'EXPECTED_L11_SERVICE_SHA256',
            'TD121_L11_PACKAGE_PATH', 'EXPECTED_TD121_L11_PACKAGE_SHA256',
            'TD121_L11_PROOF_PATH', 'EXPECTED_TD121_L11_PROOF_SHA256',
            'LAY_L11_RECEIPT', 'EXPECTED_LAY_L11_RECEIPT_SHA256',
            'LAY_L2_LEXICAL_PHASE_MEMORY', 'LAY_L2_PACKAGE',
            'EXPECTED_LAY_L2_PACKAGE_SHA256', 'LAY_L2_PRODUCTIVE_V1_PACKAGE',
            'EXPECTED_LAY_L2_PRODUCTIVE_V1_PACKAGE_SHA256',
            'TD121_PRODUCTIVE_V90_RECOVERY_PATH',
            'EXPECTED_TD121_PRODUCTIVE_V90_RECOVERY_SHA256',
            'LAY_L2_V13_DAFSA', 'EXPECTED_LAY_L2_V13_DAFSA_SHA256')
        assert not [name for name in absent_names if name in os.environ], os.environ.keys()
        assert not any(Path('/tmp/deps').iterdir()), 'dependency files visible in absent profile'
    assert DAEMON_BIN.is_file() and LOADER.is_file()
    assert SEED_BIN.is_file() and os.access(SEED_BIN, os.X_OK)
    assert not os.environ.get('DISPLAY') and not os.environ.get('WAYLAND_DISPLAY')
    assert not os.environ.get('LD_LIBRARY_PATH') and not os.environ.get('LD_PRELOAD')
    session_address = os.environ['DBUS_SESSION_BUS_ADDRESS']
    assert session_address.startswith(('unix:path=/tmp/', 'unix:abstract='))
    for name in ('runtime', 'cache', 'config', 'data', 'usage', 'no-models'):
        (ROOT / name).mkdir(mode=0o700, exist_ok=True)
    if STARTUP_PROOF_PROFILE == 'absent':
        assert not any((ROOT / 'no-models').rglob('*')), 'model files visible in absent profile'
    ibus_address = 'unix:path=/tmp/proof/ibus.sock'
    runtime_environment = {
        'IBUS_ADDRESS': ibus_address,
        'IBUS_COMPONENT_PATH': '/tmp/proof/component',
        'LAY_CONFIG_PATH': '/tmp/proof/config.json',
        'LAY_IBUS_TRACE_PATH': '/tmp/proof/ibus-engine-trace.jsonl',
        'LAY_L11_SOCKET': '/tmp/proof/runtime/l11.sock',
        'LAY_NANDA_WORD_USAGE_EVENTS': '/tmp/proof/usage/events.jsonl',
        'LAY_NANDA_WORD_USAGE_COUNTS': '/tmp/proof/usage/counts.json',
        'LAY_NANDA_WORD_USAGE_FEEDBACK_COUNTS': '/tmp/proof/usage/feedback.json',
        'LAY_NANDA_USAGE_PRIOR': '/tmp/proof/usage/prior.json',
    }
    if STARTUP_PROOF_PROFILE != 'absent':
        runtime_environment['LAY_L11_MODEL_DIR'] = str(Path(os.environ['LAY_L11_RECEIPT']).parent)
    os.environ.update(runtime_environment)
    receipt['identity'] = {
        'candidate_path': str(CANDIDATE), 'candidate_sha256': sha256(CANDIDATE),
        'copied_daemon_sha256': sha256(DAEMON_BIN),
        'copied_loader_sha256': sha256(LOADER),
        'remote_system_daemon_sha256': sha256(SYSTEM_DAEMON),
        'remote_system_daemon_not_used': True,
        'seed_engine_path': str(SEED_BIN), 'seed_engine_sha256': sha256(SEED_BIN),
        'seed_is_initial_foreign_profile_only': True,
        'lay_component_sha256': sha256(ROOT / 'component/lay.xml'),
        'seed_component_sha256': sha256(ROOT / 'component/seed-simple.xml'),
        'private_config_sha256': sha256(ROOT / 'config.json'),
        'authority_files': authority_files,
    }
    receipt['isolation'] = {
        'inherited_home': os.environ.get('HOME'),
        'session_bus_address': session_address,
        'ibus_address': ibus_address,
        'namespace_ids': {name: os.readlink('/proc/self/ns/' + name)
                          for name in ('pid', 'net', 'ipc', 'mnt')},
        'packages_absent': STARTUP_PROOF_PROFILE == 'absent',
    }
    daemon_log = (ROOT / 'private-daemon-and-components.stderr').open('x', encoding='utf-8')
    daemon = subprocess.Popen([
        str(LOADER), '--library-path', str(LIBS), str(DAEMON_BIN), '--single',
        '--panel=disable', '--emoji-extension=disable', '--config=disable',
        '--cache=none', '--address=' + ibus_address,
    ], stdin=subprocess.DEVNULL, stdout=daemon_log, stderr=daemon_log)
    wait_until(lambda: (ROOT / 'ibus.sock').exists() and daemon.poll() is None,
               'private copied IBus startup', 5.0)
    IBus.init()
    ibus_bus = IBus.Bus.new()
    assert ibus_bus.is_connected()
    connection = ibus_bus.get_connection()
    connection.set_exit_on_close(False)
    session_connection = Gio.bus_get_sync(Gio.BusType.SESSION, None)
    session_connection.set_exit_on_close(False)
    engines = {desc.get_name() for desc in ibus_bus.list_engines()}
    assert {SEED_ENGINE, US_ENGINE, RU_ENGINE}.issubset(engines), sorted(engines)
    bootstrap = Client('td121-bootstrap-seed')
    bootstrap.focus_in()
    select_engine(SEED_ENGINE)
    receipt['global_mode'] = bool(ibus_bus.get_use_global_engine())
    assert receipt['global_mode'] is True
    behavior_started_ns = time.monotonic_ns()
    subscribe_barriers(connection)
    if SCENARIO_SET == 'terminal-delivery':
        run_terminal_delivery_cases()
        assert len(receipt['cases']) == 4, receipt['cases']
    elif SCENARIO_SET in ('first-word', 'first-word-us', 'first-word-ru'):
        run_manual_toggle_cases(first_word=True)
        expected_cases = (['first_word_us', 'first_word_ru'] if SCENARIO_SET == 'first-word'
                          else ['first_word_' + SCENARIO_SET.rsplit('-', 1)[1]])
        assert [row['case'] for row in receipt['cases']] == expected_cases, receipt['cases']
    elif SCENARIO_SET == 'fresh-preedit':
        run_fresh_preedit_case()
    elif SCENARIO_SET == 'startup-only':
        run_startup_only_case()
    elif SCENARIO_SET == 'packages-absent-literal':
        assert STARTUP_PROOF_PROFILE == 'absent'
        run_packages_absent_literal_case()
    elif SCENARIO_SET == 'manual-toggle':
        run_manual_toggle_cases()
        assert len(receipt['cases']) == 3, receipt['cases']
    elif SCENARIO_SET == 'lifecycle':
        run_lifecycle_cases()
        assert [case['case'] for case in receipt['cases']] == [
            'same_engine_refocus_via_dummy_context_discards_word',
            'refocus_terminal_content_type_drops_word_not_context',
            'refocus_reset_drops_word_not_context',
        ], receipt['cases']
    else:
        run_cases()
    receipt['behavior'] = {
        'verdict': ('PASS_3_LIFECYCLE_CASES' if SCENARIO_SET == 'lifecycle'
                    else 'PASS_4_TERMINAL_DELIVERY_CASES' if SCENARIO_SET == 'terminal-delivery'
                    else 'PASS_3_MANUAL_CASES' if SCENARIO_SET == 'manual-toggle'
                    else 'PASS_2_FIRST_WORD_CASES' if SCENARIO_SET == 'first-word'
                    else 'PASS_1_FIRST_WORD_CASE' if SCENARIO_SET in ('first-word-us', 'first-word-ru')
                    else 'PASS_1_STARTUP_CASE' if SCENARIO_SET in ('fresh-preedit', 'startup-only', 'packages-absent-literal')
                    else 'PASS_5_CASES_AUTHORITY_ON'),
        'completed_cases': len(receipt['cases']),
        'started_monotonic_ns': behavior_started_ns,
        'finished_monotonic_ns': time.monotonic_ns(),
    }
    receipt['candidate_processes'] = candidate_pids(include_status=True)
    receipt['candidate_memory_interpretation'] = (
        'single post-verdict process samples with kernel VmHWM; '
        'not a production RSS distribution')
    assert len(receipt['candidate_processes']) == 1, receipt['candidate_processes']
    receipt['status'] = COMPLETED_STATUS
except Exception as error:
    receipt['status'] = 'FAILED'
    receipt['error'] = f'{type(error).__name__}: {error}'
    receipt['behavior'] = {
        'verdict': 'FAILED_FROZEN_BEFORE_TRACE_DRAIN',
        'finished_monotonic_ns': time.monotonic_ns(),
        'completed_cases': len(receipt['cases']),
    }
    traceback.print_exc()
finally:
    signal.alarm(0)
    trace_path = ROOT / 'ibus-engine-trace.jsonl'
    trace_before = {
        'exists': trace_path.exists(),
        'bytes': trace_path.stat().st_size if trace_path.exists() else 0,
    }
    drain_started_ns = time.monotonic_ns()
    candidate_alive_for_drain = bool(candidate_pids())
    if candidate_alive_for_drain:
        # Artifact-only drain for the asynchronous 1000 ms debug-log batch.
        # Behavior verdict and timestamps above are immutable before this wait.
        time.sleep(1.25)
    receipt['post_verdict_trace_drain'] = {
        'purpose': 'artifact_flush_only',
        'can_change_behavior_verdict': False,
        'candidate_alive': candidate_alive_for_drain,
        'started_monotonic_ns': drain_started_ns,
        'finished_monotonic_ns': time.monotonic_ns(),
        'trace_before': trace_before,
        'trace_after': {
            'exists': trace_path.exists(),
            'bytes': trace_path.stat().st_size if trace_path.exists() else 0,
        },
        'absence_interpretation': 'UNKNOWN_NOT_NO_CALLBACK',
    }
    for client in reversed(contexts):
        try:
            client.focus_out()
        except Exception:
            pass
    if session_connection is not None:
        try:
            session_connection.close_sync(None)
        except Exception:
            pass
    if ibus_bus is not None:
        try:
            ibus_bus.get_connection().close_sync(None)
        except Exception:
            pass
    before_cleanup = candidate_pids()
    if daemon is not None:
        daemon.terminate()
        try:
            daemon.wait(timeout=3)
        except subprocess.TimeoutExpired:
            daemon.kill()
            daemon.wait(timeout=2)
        cleanup_deadline = time.monotonic() + 2
        while candidate_pids() and time.monotonic() < cleanup_deadline:
            time.sleep(0.01)
        remaining = candidate_pids()
        receipt['cleanup'] = {
            'private_daemon_pid': daemon.pid,
            'private_daemon_reaped': daemon.poll() is not None,
            'private_daemon_returncode': daemon.returncode,
            'candidate_processes_before_daemon_stop': before_cleanup,
            'candidate_processes_remaining': remaining,
        }
        if receipt['status'] == COMPLETED_STATUS and remaining:
            receipt['status'] = 'FAILED_CLEANUP'
    if daemon_log is not None:
        daemon_log.close()
    if STARTUP_PROOF_PROFILE != 'legacy':
        try:
            receipt['startup_measurement'].update(startup_trace_measurement(trace_path))
        except Exception as error:
            status_before_measurement = receipt['status']
            if status_before_measurement == COMPLETED_STATUS:
                receipt['status'] = 'FAILED_STARTUP_MEASUREMENT'
            receipt['startup_measurement_status_before_error'] = status_before_measurement
            receipt['startup_measurement_error'] = f'{type(error).__name__}: {error}'
    receipt['elapsed_s'] = time.monotonic() - started
    for handle in streams.values():
        handle.close()
    artifact_names = ['actual-input.jsonl', 'commit-output.jsonl',
                      'client-projection.jsonl',
                      'internal-observations.jsonl', 'ibus-engine-trace.jsonl',
                      'private-daemon-and-components.stderr']
    receipt['artifacts'] = {
        name: {'sha256': sha256(ROOT / name), 'bytes': (ROOT / name).stat().st_size}
        for name in artifact_names if (ROOT / name).exists()
    }
    (ROOT / 'receipt.json').write_text(
        json.dumps(receipt, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'status': receipt['status'], 'cases': len(receipt['cases']),
                      'cleanup': receipt.get('cleanup')}, ensure_ascii=False), flush=True)
if receipt['status'] != COMPLETED_STATUS:
    raise SystemExit(1)
