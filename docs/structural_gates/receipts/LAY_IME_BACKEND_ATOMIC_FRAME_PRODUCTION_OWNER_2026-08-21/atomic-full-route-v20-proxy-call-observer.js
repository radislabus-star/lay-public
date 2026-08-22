/* eslint camelcase: ["error", {properties: "never"}] */

import Clutter from 'gi://Clutter';
import GLib from 'gi://GLib';

import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as Scripting from 'resource:///org/gnome/shell/ui/scripting.js';

const ENGINE_NAME = 'lay-ime-ru';
const HOT_SAMPLES = 512;
const KEY_A = 30;
const KEY_S = 31;

async function waitUntil(predicate, description, timeoutMs = 5000) {
    const deadline = GLib.get_monotonic_time() + timeoutMs * 1000;
    while (!predicate()) {
        if (GLib.get_monotonic_time() >= deadline)
            throw new Error(`timed out waiting for ${description}`);
        await Scripting.sleep(5);
    }
}

function percentile(sorted, index) {
    return sorted[Math.min(index, sorted.length - 1)];
}

async function typeOne(device, keycode, expectedChars) {
    const observed = Scripting.waitTestWindowTextLength(expectedChars);

    // Arm the out-of-process observer before entering the measured route.
    await Scripting.sleep(1);
    const started = GLib.get_monotonic_time();
    device.notify_key(started, keycode, Clutter.KeyState.PRESSED);
    const [changedAt, text] = await observed;
    device.notify_key(GLib.get_monotonic_time(), keycode,
        Clutter.KeyState.RELEASED);

    // Keep release settlement out of the next key's measured interval.
    await Scripting.sleep(1);
    return {latencyUs: Number(changedAt) - started, text};
}

function installAtomicRouteObservers(inputMethod) {
    const adapter = inputMethod._atomic;
    const originalEnqueue = adapter.enqueue.bind(adapter);
    const originalFreezeLease = adapter._delegates.freezeLease;
    const originalInvokeAtomic = adapter._delegates.invokeAtomic;
    let enqueueObserved = false;
    let freezeObserved = false;
    let invokeObserved = false;

    function proxyField(context, getter) {
        try {
            return context[getter]();
        } catch {
            return 'unavailable';
        }
    }

    function errorField(error, field) {
        const value = error?.[field];
        if (value === null || value === undefined)
            return 'unavailable';
        return String(value).replaceAll('\n', '\\n');
    }

    adapter.enqueue = (...args) => {
        const first = !enqueueObserved;
        if (first) {
            enqueueObserved = true;
            log(`LAY_ATOMIC_ROUTE hop=enqueue state=${args[3]} ` +
                `context=${inputMethod._context !== null} ` +
                `source=${inputMethod._currentSource !== null} ` +
                `enabled=${adapter._enabled}`);
        }
        const result = originalEnqueue(...args);
        if (first)
            log(`LAY_ATOMIC_ROUTE hop=enqueue-result result=${result}`);
        return result;
    };

    adapter._delegates.freezeLease = lease => {
        try {
            const result = originalFreezeLease(lease);
            if (!freezeObserved) {
                freezeObserved = true;
                log(`LAY_ATOMIC_ROUTE hop=freeze-lease result=${result}`);
            }
            return result;
        } catch (error) {
            if (!freezeObserved) {
                freezeObserved = true;
                log(`LAY_ATOMIC_ROUTE hop=freeze-lease error=${error.message}`);
            }
            throw error;
        }
    };

    adapter._delegates.invokeAtomic = (...args) => {
        const first = !invokeObserved;
        const [context, request, timeout] = args;
        const started = GLib.get_monotonic_time();
        if (first) {
            invokeObserved = true;
            log(`LAY_ATOMIC_ROUTE hop=invoke-start ` +
                `name=${proxyField(context, 'get_name')} ` +
                `path=${proxyField(context, 'get_object_path')} ` +
                `interface=${proxyField(context, 'get_interface_name')} ` +
                `request_type=${request.get_type_string()} ` +
                `timeout_ms=${timeout}`);
        }

        let promise;
        try {
            promise = originalInvokeAtomic(...args);
        } catch (error) {
            if (first) {
                log(`LAY_ATOMIC_ROUTE hop=invoke-sync-reject ` +
                    `elapsed_us=${GLib.get_monotonic_time() - started} ` +
                    `domain=${errorField(error, 'domain')} ` +
                    `code=${errorField(error, 'code')} ` +
                    `message=${errorField(error, 'message')}`);
            }
            throw error;
        }

        if (first) {
            promise.then(reply => {
                log(`LAY_ATOMIC_ROUTE hop=invoke-resolve ` +
                    `elapsed_us=${GLib.get_monotonic_time() - started} ` +
                    `reply_type=${reply.get_type_string()}`);
            }, error => {
                log(`LAY_ATOMIC_ROUTE hop=invoke-reject ` +
                    `elapsed_us=${GLib.get_monotonic_time() - started} ` +
                    `domain=${errorField(error, 'domain')} ` +
                    `code=${errorField(error, 'code')} ` +
                    `message=${errorField(error, 'message')}`);
            });
        }
        return promise;
    };

    log(`LAY_ATOMIC_ROUTE hop=pre-event ` +
        `context=${inputMethod._context !== null} ` +
        `source=${inputMethod._currentSource !== null} ` +
        `enabled=${adapter._enabled} queue=${adapter._queue.length} ` +
        `in_flight=${adapter._inFlight !== null}`);
}

/** Run the real isolated atomic input route. */
export async function run() {
    await Scripting.disableHelperAutoExit();
    Main.overview.hide();
    await Scripting.waitLeisure();

    await Scripting.createTestWindow({
        width: 640,
        height: 240,
        textInput: true,
    });
    await Scripting.waitTestWindows();

    await waitUntil(() => Main.inputMethod?._context,
        'GNOME Shell IBus input context');
    await waitUntil(() => Main.inputMethod.currentFocus !== null,
        'Wayland text-input focus');
    await waitUntil(() => Main.inputMethod.getSurroundingText()[0] !== null,
        'exact surrounding snapshot');
    if (!Main.inputMethod._ibus.set_global_engine(ENGINE_NAME))
        throw new Error(`failed to select global IBus engine ${ENGINE_NAME}`);
    await waitUntil(() =>
        Main.inputMethod._context.get_engine()?.get_name() === ENGINE_NAME,
    'real Lay IBus engine selection');

    installAtomicRouteObservers(Main.inputMethod);

    const seat = global.stage.context.get_backend().get_default_seat();
    const device = seat.create_virtual_device(
        Clutter.InputDeviceType.KEYBOARD_DEVICE);

    const first = await typeOne(device, KEY_A, 1);
    if (first.text !== 'ф')
        throw new Error(`first real commit mismatch: ${first.text}`);

    const second = await typeOne(device, KEY_S, 2);
    if (second.text !== 'фы')
        throw new Error(`second real commit mismatch: ${second.text}`);

    const latencies = [];
    let finalText = second.text;
    for (let i = 0; i < HOT_SAMPLES; i++) {
        const sample = await typeOne(device, KEY_A, i + 3);
        latencies.push(sample.latencyUs);
        finalText = sample.text;
    }

    const expected = `фы${'ф'.repeat(HOT_SAMPLES)}`;
    if (finalText !== expected)
        throw new Error(`duplicate or missing native mutation: ${[...finalText].length}`);

    latencies.sort((a, b) => a - b);
    const p50 = percentile(latencies, 256);
    const p99 = percentile(latencies, 507);
    const max = percentile(latencies, 511);
    log(`LAY_ATOMIC_INTEGRATED samples=${HOT_SAMPLES} p50=${p50}us ` +
        `p99=${p99}us max=${max}us text_chars=${[...finalText].length}`);

    if (p99 > 5000)
        throw new Error(`integrated p99 exceeds 5 ms: ${p99} us`);
    if (max >= 8000)
        throw new Error(`integrated max is not below 8 ms: ${max} us`);
}
