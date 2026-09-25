#define _GNU_SOURCE
#include <dlfcn.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

enum {
    LAY_GDK_KEY_RELEASE = 9,
    LAY_GDK_CONTROL_MASK = 1u << 2,
    LAY_GDK_MOD1_MASK = 1u << 3,
    LAY_GDK_SUPER_MASK = 1u << 26,
    LAY_GDK_HYPER_MASK = 1u << 27,
    LAY_GDK_META_MASK = 1u << 28,
    LAY_IBUS_CAP_EXACT_SURROUNDING_REFRESH = 1u << 30,
};

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
} LayGdkEventKey;

static _Thread_local unsigned request_count;

void ibus_input_context_set_capabilities(void *context, unsigned capabilities) {
    typedef void (*SetCapabilities)(void *, unsigned);
    static SetCapabilities real_set_capabilities;
    static void *ibus_library;

    if (!real_set_capabilities) {
        real_set_capabilities =
            (SetCapabilities)dlsym(RTLD_NEXT, "ibus_input_context_set_capabilities");
    }
    if (!real_set_capabilities) {
        ibus_library = dlopen("libibus-1.0.so.5", RTLD_NOW | RTLD_NOLOAD);
        if (ibus_library) {
            real_set_capabilities = (SetCapabilities)dlsym(
                ibus_library, "ibus_input_context_set_capabilities");
        }
    }
    if (real_set_capabilities == ibus_input_context_set_capabilities) {
        real_set_capabilities = NULL;
    }
    if (!real_set_capabilities) {
        static const char error[] =
            "lay-context-notify: missing original capability setter\n";
        const ssize_t written = write(STDERR_FILENO, error, sizeof(error) - 1);
        (void)written;
        _exit(125);
    }
    real_set_capabilities(
        context, capabilities | LAY_IBUS_CAP_EXACT_SURROUNDING_REFRESH);
}

static void request_surrounding(void *context, const char *reason, unsigned keyval) {
    typedef const char *(*Name)(void *);
    typedef void (*Emit)(void *, const char *, ...);
    static Name type_name;
    static Name module_id;
    static Emit emit;
    static int resolved;

    if (!resolved) {
        type_name = (Name)dlsym(RTLD_DEFAULT, "g_type_name_from_instance");
        module_id = (Name)dlsym(RTLD_DEFAULT, "gtk_im_multicontext_get_context_id");
        emit = (Emit)dlsym(RTLD_DEFAULT, "g_signal_emit_by_name");
        resolved = 1;
    }
    if (!context || !type_name || !module_id || !emit) {
        return;
    }
    const char *type = type_name(context);
    const char *module = type ? module_id(context) : NULL;
    if (!type || strcmp(type, "GtkIMMulticontext") != 0
        || !module || strcmp(module, "ibus") != 0) {
        return;
    }
    int supplied = 0;
    emit(context, "retrieve-surrounding", &supplied);
    if (++request_count <= 96) {
        (void)dprintf(STDERR_FILENO,
            "lay-context-notify: request=%u reason=%s keyval=%u supplied=%d\n",
            request_count, reason, keyval, supplied);
    }
}

int gtk_im_context_filter_keypress(void *context, LayGdkEventKey *event) {
    typedef int (*Filter)(void *, LayGdkEventKey *);
    typedef uint32_t (*KeyvalToUnicode)(unsigned);
    static Filter real_filter;
    static KeyvalToUnicode keyval_to_unicode;
    static _Thread_local unsigned depth;

    if (!real_filter) {
        real_filter = (Filter)dlsym(RTLD_NEXT, "gtk_im_context_filter_keypress");
        keyval_to_unicode =
            (KeyvalToUnicode)dlsym(RTLD_DEFAULT, "gdk_keyval_to_unicode");
    }
    if (!real_filter) {
        static const char error[] = "lay-context-notify: missing original filter\n";
        const ssize_t written = write(STDERR_FILENO, error, sizeof(error) - 1);
        (void)written;
        _exit(125);
    }

    ++depth;
    const int handled = real_filter(context, event);
    if (depth == 1 && handled && event && keyval_to_unicode
        && event->type == LAY_GDK_KEY_RELEASE
        && !(event->state & (LAY_GDK_CONTROL_MASK | LAY_GDK_MOD1_MASK
            | LAY_GDK_SUPER_MASK | LAY_GDK_HYPER_MASK | LAY_GDK_META_MASK))) {
        const uint32_t codepoint = keyval_to_unicode(event->keyval);
        if (codepoint >= 0x20 && codepoint != 0x7f) {
            request_surrounding(context, "release", event->keyval);
        }
    }
    --depth;
    return handled;
}

void gtk_im_context_reset(void *context) {
    typedef void (*Reset)(void *);
    static Reset real_reset;
    static _Thread_local unsigned depth;

    if (!real_reset) {
        real_reset = (Reset)dlsym(RTLD_NEXT, "gtk_im_context_reset");
    }
    if (!real_reset) {
        static const char error[] = "lay-context-notify: missing original reset\n";
        const ssize_t written = write(STDERR_FILENO, error, sizeof(error) - 1);
        (void)written;
        _exit(125);
    }

    ++depth;
    real_reset(context);
    if (depth == 1) {
        request_surrounding(context, "reset", 0);
    }
    --depth;
}
