/* Real IBusText/GObject ownership and the actual upstream InputContext setter.
 * Only the context wrapper and outgoing D-Bus call are replaced. */
#include <ibus.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
typedef struct {
    IBusText *surrounding_text;
    guint surrounding_cursor_pos, selection_anchor_pos;
    gboolean needs_surrounding_text;
} RecordingContext;
typedef RecordingContext BusEngineProxy;
typedef struct {
    RecordingContext *ibuscontext;
    IBusText *surrounding_text;
    guint surrounding_cursor_pos, surrounding_anchor_pos;
} IBusWaylandIMPrivate;
typedef IBusWaylandIMPrivate IBusWaylandIM;
struct zwp_input_method_context_union { int unused; };
static int calls, resets;
static void record_call(GDBusProxy *p, const gchar *method, GVariant *v,
                        GDBusCallFlags f, gint timeout, GCancellable *c,
                        GAsyncReadyCallback cb, gpointer data) {
    if (!strcmp(method, "Reset")) {
        g_assert_null(v);
        resets++;
        return;
    }
    g_assert_cmpstr(method, ==, "SetSurroundingText");
    calls++;
    g_variant_unref(g_variant_ref_sink(v));
}
#undef IBUS_IS_INPUT_CONTEXT
#define IBUS_IS_INPUT_CONTEXT(p) ((p) != NULL)
#define BUS_IS_ENGINE_PROXY(p) ((p) != NULL)
#define IBUS_INPUT_CONTEXT_GET_PRIVATE(p) (p)
#define IBusInputContext RecordingContext
#define IBusInputContextPrivate RecordingContext
#define ibus_input_context_set_surrounding_text recording_set_surrounding_text
#define ibus_input_context_needs_surrounding_text(p) ((p)->needs_surrounding_text)
#define g_dbus_proxy_call record_call
#define ENABLE_SURROUNDING 1
#define IBUS_IS_WAYLAND_IM(p) ((p) != NULL)
#define IBUS_WAYLAND_IM(p) ((IBusWaylandIM *)(p))
#define ibus_wayland_im_get_instance_private(p) (p)
#include "actual-lifetime-functions.h"

int main(void) {
    RecordingContext c = {.needs_surrounding_text = TRUE};
    IBusWaylandIMPrivate p = {.ibuscontext = &c};
    handle_surrounding_text(&p, NULL, "я аб", 5, 0);
    for (int i = 0; i < 32; i++) {
        _context_require_surrounding_text_cb(&c, &p);
        g_assert(IBUS_IS_TEXT(p.surrounding_text));
        g_assert(IBUS_IS_TEXT(c.surrounding_text));
        g_assert_cmpstr(p.surrounding_text->text, ==, "я аб");
        g_assert_cmpuint(c.surrounding_cursor_pos, ==, 3);
        g_assert_cmpuint(c.selection_anchor_pos, ==, 0);
    }
    handle_surrounding_text(&p, NULL, "привет", 12, 12);
    _context_require_surrounding_text_cb(&c, &p);
    g_assert_cmpstr(c.surrounding_text->text, ==, "привет");
    g_clear_object(&p.surrounding_text);
    g_clear_object(&c.surrounding_text);
    int before = calls;
    _context_require_surrounding_text_cb(&c, &p);
    g_assert_cmpint(calls, ==, before);
    BusEngineProxy proxy = {0};
    proxy.surrounding_text = g_object_ref_sink(ibus_text_new_from_string("я аб"));
    IBusText *weak = proxy.surrounding_text;
    g_object_add_weak_pointer(G_OBJECT(weak), (gpointer *)&weak);
    bus_engine_proxy_reset(&proxy);
    g_assert_null(proxy.surrounding_text);
    g_assert_null(weak);
    bus_engine_proxy_reset(&proxy);
    g_assert_cmpint(resets, ==, 2);
    g_assert_cmpint(calls, ==, before);
    puts("PASS real-object repeated Require, callback replacement, Unicode anchor, cleared cache, Reset releases sole reference without cached replay");
    return 0;
}
