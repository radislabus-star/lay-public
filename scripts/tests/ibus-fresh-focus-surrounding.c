/* Executes extracted upstream functions. Transport is recorded, not emulated
 * as a second implementation of their decisions. Native acceptance is separate. */
#include <glib.h>
#include <stdint.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>

typedef struct { char *text; } IBusText;
typedef IBusText IBusSerializable;
typedef struct { int needs; } IBusInputContext;
typedef struct {
    int has_focus, has_focus_id, has_active_surrounding_text;
    char *object_path, *client;
    IBusText *surrounding_text;
    guint surrounding_cursor_pos, selection_anchor_pos;
} BusEngineProxy;
typedef struct {
    IBusInputContext *ibuscontext;
    IBusText *surrounding_text;
    guint surrounding_cursor_pos, surrounding_anchor_pos;
} IBusWaylandIMPrivate;
typedef IBusWaylandIMPrivate IBusWaylandIM;
typedef void GDBusProxy;
struct zwp_input_method_context_union { int unused; };
static int calls, resets;
static guint delivered_cursor, delivered_anchor;
static char *delivered_text;
#define ENABLE_SURROUNDING 1
#define BUS_IS_ENGINE_PROXY(p) ((p) != NULL)
#define IBUS_IS_WAYLAND_IM(p) ((p) != NULL)
#define IBUS_WAYLAND_IM(p) ((IBusWaylandIM *)(p))
#define G_DBUS_CALL_FLAGS_NONE 0
#define g_object_ref_sink(p) (p)
#define g_object_unref(p) ((void)(p))
#define g_clear_object(p) (*(p) = NULL)
#define g_signal_emit(...) ((void)0)
#define ibus_wayland_im_get_instance_private(p) (p)
#define ibus_input_context_needs_surrounding_text(p) ((p)->needs)
static IBusText *ibus_text_new_from_string(const char *s) {
    IBusText *t = g_new0(IBusText, 1); t->text = g_strdup(s); return t;
}
static GVariant *ibus_serializable_serialize(IBusSerializable *t) {
    return g_variant_new_string(t->text);
}
static void g_dbus_proxy_call(GDBusProxy *p, const char *method,
                             GVariant *v, int flags, int timeout,
                             void *cancellable, void *callback, void *data) {
    if (!strcmp(method, "SetSurroundingText")) {
        GVariant *text;
        calls++;
        g_variant_get(v, "(vuu)", &text, &delivered_cursor, &delivered_anchor);
        delivered_text = g_variant_dup_string(text, NULL);
        g_variant_unref(text);
    }
    if (!strcmp(method, "Reset")) resets++;
    if (v) g_variant_unref(g_variant_ref_sink(v));
}
static void ibus_input_context_set_surrounding_text(IBusInputContext *c,
                                                   IBusText *t, guint cursor,
                                                   guint anchor) {
    calls++; delivered_cursor = cursor; delivered_anchor = anchor;
    delivered_text = g_strdup(t->text);
}
#include "actual-upstream-functions.h"

int main(int argc, char **argv) {
    g_assert(argc == 2);
    BusEngineProxy e = {0};
    IBusText text = {"hello"};
    IBusInputContext a = {1}, b = {1};
    IBusWaylandIMPrivate p = {0};
    int id = atoi(argv[1]), expected = 0;
    switch (id) {
    case 1: /* Real focus-out/in must not suppress the new field's same tuple. */
    case 2: /* Redundant FocusIn for the same live context is still deduplicated. */
    case 3: /* A changed focus ID also ends the old tuple's scope. */
        bus_engine_proxy_focus_in(&e, "/field/a", "wayland");
        bus_engine_proxy_set_surrounding_text(&e, &text, 5, 5);
        if (id == 1) bus_engine_proxy_focus_out(&e, "/field/a");
        bus_engine_proxy_focus_in(&e, id == 2 ? "/field/a" : "/field/b", "wayland");
        bus_engine_proxy_set_surrounding_text(&e, &text, 5, 5);
        expected = id == 2 ? 1 : 2;
        break;
    case 4: /* Actual callback arrived while async context creation was pending. */
    case 5: /* Context exists but its Require signal has not arrived yet. */
        a.needs = 0;
        p.ibuscontext = id == 4 ? NULL : &a;
        handle_surrounding_text(&p, NULL, "я аб", 5, 0);
        g_assert(calls == 0);
        p.ibuscontext = &a; a.needs = 1;
        _context_require_surrounding_text_cb(&a, &p);
        expected = 1;
        break;
    case 6: /* Live callback preserves Unicode character cursor and selection. */
        p.ibuscontext = &a;
        handle_surrounding_text(&p, NULL, "я аб", 5, 0);
        expected = 1;
        break;
    case 7: /* No callback for this activation grants no cached observation. */
        p.ibuscontext = &a;
        _context_require_surrounding_text_cb(&a, &p);
        expected = 0;
        break;
    case 8: /* A signal belonging to a different context cannot forward a tuple. */
        p.ibuscontext = &a;
        a.needs = 0;
        handle_surrounding_text(&p, NULL, "я аб", 5, 0);
        _context_require_surrounding_text_cb(&b, &p);
        expected = 0;
        break;
    case 9: /* Reset retires delivery; an actual equal tuple must arrive again. */
        bus_engine_proxy_set_surrounding_text(&e, &text, 5, 5);
        bus_engine_proxy_reset(&e);
        g_assert(calls == 1 && resets == 1);
        bus_engine_proxy_set_surrounding_text(&e, &text, 5, 5);
        bus_engine_proxy_set_surrounding_text(&e, &text, 5, 5);
        expected = 2; /* The repeated post-Reset callback is still deduplicated. */
        break;
    case 10: /* Repeated Reset never republishes an old cached observation. */
        bus_engine_proxy_set_surrounding_text(&e, &text, 5, 5);
        bus_engine_proxy_reset(&e);
        bus_engine_proxy_reset(&e);
        g_assert(resets == 2);
        expected = e.surrounding_text == NULL ? 1 : -1;
        break;
    case 11: /* Empty Reset then actual Unicode/selection tuple stays exact. */
        bus_engine_proxy_reset(&e);
        g_assert(calls == 0 && resets == 1);
        text.text = "я аб";
        bus_engine_proxy_set_surrounding_text(&e, &text, 3, 0);
        g_assert(delivered_cursor == 3 && delivered_anchor == 0 &&
                 !strcmp(delivered_text, "я аб"));
        expected = 1;
        break;
    case 12: { /* Reset invalidates only its engine's delivery cache. */
        BusEngineProxy other = {0};
        bus_engine_proxy_set_surrounding_text(&e, &text, 5, 5);
        bus_engine_proxy_set_surrounding_text(&other, &text, 5, 5);
        bus_engine_proxy_reset(&e);
        bus_engine_proxy_set_surrounding_text(&other, &text, 5, 5);
        bus_engine_proxy_set_surrounding_text(&e, &text, 5, 5);
        g_assert(resets == 1);
        expected = 3;
        break;
    }
    default: return 2;
    }
    gboolean ok = calls == expected;
    if (id >= 4 && id <= 6 && calls)
        ok &= delivered_cursor == 3 && delivered_anchor == 0 &&
              !strcmp(delivered_text, "я аб");
    printf("case=%d status=%s calls=%d expected=%d cursor=%u anchor=%u\n",
           id, ok ? "PASS" : "FAIL", calls, expected,
           delivered_cursor, delivered_anchor);
    return ok ? 0 : 1;
}
