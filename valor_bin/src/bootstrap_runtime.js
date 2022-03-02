"use strict";

((window) => {
    const {
        ObjectDefineProperties,
    } = window.__bootstrap.primordials;
    const core = Deno.core;
    const location = window.__bootstrap.location;
    const timers = window.__bootstrap.timers;
    const Console = window.__bootstrap.console.Console;
    // const fetch = window.__bootstrap.fetch;

    const util = {
        writable(value) {
            return { value, writable: true, enumerable: true, configurable: true, }
        },
        nonEnumerable(value) {
            return { value, writable: true, enumerable: false, configurable: true, }
        },
        readOnly(value) {
            return { value, writable: false, enumerable: true, configurable: true, }
        },
    }

    const globalScope = {
        Location: location.locationConstructorDescriptor,
        location: location.locationDescriptor,
        window: util.readOnly(globalThis),
        self: util.writable(globalThis),
        console: util.nonEnumerable(
            new Console((msg, level) => core.print(msg, level > 1))
        ),
        // fetch: util.writable(fetch.fetch),
        setInterval: util.writable(timers.setInterval),
        setTimeout: util.writable(timers.setTimeout),
        clearInterval: util.writable(timers.clearInterval),
        clearTimeout: util.writable(timers.clearTimeout),
        // Request: util.nonEnumerable(fetch.Request),
        // Response: util.nonEnumerable(fetch.Response),
    };
    ObjectDefineProperties(globalThis, globalScope);

    const consoleFromV8 = window.console;
    const wrapConsole = window.__bootstrap.console.wrapConsole;
    const consoleFromDeno = globalThis.console;
    wrapConsole(consoleFromDeno, consoleFromV8);

    delete globalThis.__bootstrap;
    delete globalThis.bootstrap

    // runtime start
    core.setMacrotaskCallback(timers.handleTimerMacrotask);
    // core.setWasmStreamingCallback(fetch.handleWasmStreaming);
})(this);

(async () => {
    console.log('fetching...', location);
    // let code = await fetch('./test_plugin/test_plugin_bg.wasm');
    // console.log('code> ', code);
    //code = await code.arrayBuffer();
    //const wasm = (await WebAssembly.instantiate(code, {})).instance;
    //console.log(wasm.on_create());
})();
