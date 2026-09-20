import os
import pathlib
import subprocess
import tempfile
import textwrap
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
ADAPTER = ROOT / "scripts" / "compat" / "firefox-ibus-reset-notify.c"
LAUNCHER = ROOT / "scripts" / "compat" / "lay-firefox"


FAKE_GTK = r"""
#include <stdarg.h>
#include <stdint.h>
#include <string.h>

static int handled = 1;
static int ibus_module = 1;
static int multicontext = 1;
static int filter_calls;
static int reset_calls;
static int retrieve_calls;
static int capability_calls;
static unsigned last_capabilities;

void fake_gtk_configure(int next_handled, int next_ibus, int next_multicontext) {
    handled = next_handled;
    ibus_module = next_ibus;
    multicontext = next_multicontext;
    filter_calls = 0;
    reset_calls = 0;
    retrieve_calls = 0;
    capability_calls = 0;
    last_capabilities = 0;
}

int fake_gtk_filter_calls(void) { return filter_calls; }
int fake_gtk_reset_calls(void) { return reset_calls; }
int fake_gtk_retrieve_calls(void) { return retrieve_calls; }
int fake_ibus_capability_calls(void) { return capability_calls; }
unsigned fake_ibus_last_capabilities(void) { return last_capabilities; }

void ibus_input_context_set_capabilities(void *context, unsigned capabilities) {
    (void)context;
    ++capability_calls;
    last_capabilities = capabilities;
}

int gtk_im_context_filter_keypress(void *context, void *event) {
    (void)context;
    (void)event;
    ++filter_calls;
    return handled;
}

void gtk_im_context_reset(void *context) {
    (void)context;
    ++reset_calls;
}

const char *g_type_name_from_instance(void *context) {
    (void)context;
    return multicontext ? "GtkIMMulticontext" : "GtkIMContextSimple";
}

const char *gtk_im_multicontext_get_context_id(void *context) {
    (void)context;
    return ibus_module ? "ibus" : "wayland";
}

void g_signal_emit_by_name(void *context, const char *signal, ...) {
    (void)context;
    if (strcmp(signal, "retrieve-surrounding") != 0) {
        return;
    }
    va_list args;
    va_start(args, signal);
    int *supplied = va_arg(args, int *);
    va_end(args);
    ++retrieve_calls;
    if (supplied) {
        *supplied = 1;
    }
}

uint32_t gdk_keyval_to_unicode(unsigned keyval) {
    if (keyval >= 0x20u && keyval <= 0x7eu) {
        return keyval;
    }
    return 0;
}
"""


DRIVER = r"""
#include <stdint.h>
#include <stdio.h>

enum { KEY_PRESS = 8, KEY_RELEASE = 9, CONTROL_MASK = 1u << 2 };

typedef struct {
    int type;
    void *window;
    int8_t send_event;
    uint32_t time;
    unsigned state;
    unsigned keyval;
    int length;
    char *string;
    uint16_t hardware_keycode;
    uint8_t group;
    unsigned is_modifier : 1;
} Event;

extern void fake_gtk_configure(int, int, int);
extern int fake_gtk_filter_calls(void);
extern int fake_gtk_reset_calls(void);
extern int fake_gtk_retrieve_calls(void);
extern int fake_ibus_capability_calls(void);
extern unsigned fake_ibus_last_capabilities(void);
extern int gtk_im_context_filter_keypress(void *, Event *);
extern void gtk_im_context_reset(void *);
extern void ibus_input_context_set_capabilities(void *, unsigned);

static int failures;

static void expect(const char *name, int actual, int wanted) {
    if (actual != wanted) {
        fprintf(stderr, "%s: got %d, wanted %d\n", name, actual, wanted);
        ++failures;
    }
}

static void run_key(const char *name, int type, unsigned state, unsigned keyval,
                    int handled, int ibus, int multicontext, int requests) {
    Event event = {0};
    event.type = type;
    event.state = state;
    event.keyval = keyval;
    fake_gtk_configure(handled, ibus, multicontext);
    expect(name, gtk_im_context_filter_keypress((void *)1, &event), handled);
    expect("filter calls", fake_gtk_filter_calls(), 1);
    expect("filter requests", fake_gtk_retrieve_calls(), requests);
}

static void run_reset(const char *name, int ibus, int multicontext, int requests) {
    fake_gtk_configure(1, ibus, multicontext);
    gtk_im_context_reset((void *)1);
    expect(name, fake_gtk_reset_calls(), 1);
    expect("reset requests", fake_gtk_retrieve_calls(), requests);
}

int main(void) {
    run_key("printable release", KEY_RELEASE, 0, 'a', 1, 1, 1, 1);
    run_key("press", KEY_PRESS, 0, 'a', 1, 1, 1, 0);
    run_key("unhandled", KEY_RELEASE, 0, 'a', 0, 1, 1, 0);
    run_key("command modifier", KEY_RELEASE, CONTROL_MASK, 'a', 1, 1, 1, 0);
    run_key("non-printable", KEY_RELEASE, 0, 0xff0du, 1, 1, 1, 0);
    run_key("non-ibus", KEY_RELEASE, 0, 'a', 1, 0, 1, 0);
    run_key("non-multicontext", KEY_RELEASE, 0, 'a', 1, 1, 0, 0);
    run_reset("reset", 1, 1, 1);
    run_reset("non-ibus reset", 0, 1, 0);
    run_reset("non-multicontext reset", 1, 0, 0);
    fake_gtk_configure(1, 1, 1);
    ibus_input_context_set_capabilities((void *)1, 41u);
    expect("capability calls", fake_ibus_capability_calls(), 1);
    expect("commit-only preedit capability", (int)fake_ibus_last_capabilities(),
           (int)(41u | (1u << 30)));
    return failures ? 1 : 0;
}
"""


class FirefoxCompatibilityAdapterTests(unittest.TestCase):
    def test_adapter_preserves_gtk_and_requests_only_admitted_contexts(self):
        with tempfile.TemporaryDirectory(prefix="lay-firefox-adapter-") as raw:
            directory = pathlib.Path(raw)
            fake_source = directory / "fake_gtk.c"
            driver_source = directory / "driver.c"
            fake_library = directory / "libfakegtk.so"
            adapter_library = directory / "liblay-firefox-adapter.so"
            driver = directory / "driver"
            fake_source.write_text(textwrap.dedent(FAKE_GTK), encoding="utf-8")
            driver_source.write_text(textwrap.dedent(DRIVER), encoding="utf-8")

            common = ["cc", "-std=c11", "-Wall", "-Wextra", "-Werror"]
            subprocess.run(
                [*common, "-fPIC", "-shared", str(fake_source), "-o", str(fake_library)],
                check=True,
            )
            subprocess.run(
                [*common, "-fPIC", "-shared", str(ADAPTER), "-ldl", "-o", str(adapter_library)],
                check=True,
            )
            subprocess.run(
                [
                    *common,
                    str(driver_source),
                    f"-L{directory}",
                    "-lfakegtk",
                    f"-Wl,-rpath,{directory}",
                    "-o",
                    str(driver),
                ],
                check=True,
            )
            environment = os.environ.copy()
            environment["LD_PRELOAD"] = str(adapter_library)
            subprocess.run([str(driver)], check=True, env=environment)

    def test_launcher_uses_async_direct_ibus_inside_snap(self):
        source = LAUNCHER.read_text(encoding="utf-8")
        self.assertIn("snap run --shell firefox", source)
        self.assertIn("GTK_IM_MODULE=ibus", source)
        self.assertIn('preload="$library:$LD_PRELOAD"', source)
        self.assertIn('LD_PRELOAD="$preload"', source)
        self.assertIn("unset IBUS_ENABLE_SYNC_MODE", source)
        self.assertNotIn("IBUS_ENABLE_SYNC_MODE=1", source)


if __name__ == "__main__":
    unittest.main()
