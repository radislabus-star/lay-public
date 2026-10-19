// One request, one read on KWin's main thread. No handlers, timers or input.
(function () {
    var window = workspace.activeClient;
    callDBus(layQuery.destination, layQuery.path, layQuery.interface, "Observe",
        layQuery.request,
        window ? String(window.internalId) : "",
        window ? String(window.desktopFileName) : "",
        window ? String(window.resourceClass) : "",
        window ? String(window.caption) : "");
}());
