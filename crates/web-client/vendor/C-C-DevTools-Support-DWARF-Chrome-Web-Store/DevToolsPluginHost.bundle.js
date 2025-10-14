// Copyright 2020 Google LLC. All rights reserved.
// Copyright 2020 Google LLC. All rights reserved.
const DEFAULT_MODULE_CONFIGURATIONS = [{ pathSubstitutions: [] }];

// Copyright 2022 Google LLC. All rights reserved.
function serializeWasmValue(value, buffer) {
    if (value instanceof ArrayBuffer) {
        const data = new Uint8Array(value);
        new Uint8Array(buffer).set(data);
        return data.byteLength || -1;
    }
    const view = new DataView(buffer);
    switch (value.type) {
        case 'i32':
            view.setInt32(0, value.value, true);
            return 1 /* SerializedWasmType.i32 */;
        case 'i64':
            view.setBigInt64(0, value.value, true);
            return 2 /* SerializedWasmType.i64 */;
        case 'f32':
            view.setFloat32(0, value.value, true);
            return 3 /* SerializedWasmType.f32 */;
        case 'f64':
            view.setFloat64(0, value.value, true);
            return 4 /* SerializedWasmType.f64 */;
        case 'v128':
            const [enc, a, b, c, d] = value.value.split(' ');
            view.setInt32(0, Number(a), true);
            view.setInt32(4, Number(b), true);
            view.setInt32(8, Number(c), true);
            view.setInt32(12, Number(d), true);
            return 5 /* SerializedWasmType.v128 */;
        default:
            throw new Error('cannot serialize non-numerical wasm type');
    }
}

// Copyright 2022 Google LLC. All rights reserved.
class SynchronousIOMessage {
    buffer;
    constructor(bufferSize) {
        this.buffer = new SharedArrayBuffer(bufferSize);
    }
    static serialize(value, buffer) {
        return serializeWasmValue(value, buffer);
    }
}
/* eslint-disable-next-line @typescript-eslint/no-explicit-any */
class WorkerRPC {
    nextRequestId = 0;
    channel;
    localHandler;
    requests = new Map();
    semaphore;
    constructor(channel, localHandler) {
        this.channel = channel;
        this.channel.onmessage = this.onmessage.bind(this);
        this.localHandler = localHandler;
        this.semaphore = new Int32Array(new SharedArrayBuffer(4));
    }
    sendMessage(method, ...params) {
        const requestId = this.nextRequestId++;
        const promise = new Promise((resolve, reject) => {
            this.requests.set(requestId, { resolve, reject });
        });
        this.channel.postMessage({ requestId, request: { method, params } });
        return promise;
    }
    sendMessageSync(message, method, ...params) {
        const requestId = this.nextRequestId++;
        Atomics.store(this.semaphore, 0, 0);
        this.channel.postMessage({
            requestId,
            sync_request: {
                request: { method, params },
                io_buffer: { semaphore: this.semaphore.buffer, data: message.buffer },
            },
        });
        while (Atomics.wait(this.semaphore, 0, 0) !== 'not-equal') {
        }
        const [response] = this.semaphore;
        return message.deserialize(response);
    }
    async onmessage(event) {
        if ('request' in event.data) {
            const { requestId, request } = event.data;
            try {
                const response = await this.localHandler[request.method](...request.params);
                this.channel.postMessage({ requestId, response });
            }
            catch (error) {
                this.channel.postMessage({ requestId, error: error }); // FIXME type?
            }
        }
        else if ('sync_request' in event.data) {
            /* eslint-disable-next-line @typescript-eslint/naming-convention */
            const { sync_request: { request, io_buffer } } = event.data;
            let signal = -1;
            try {
                const response = await this.localHandler[request.method](...request.params);
                signal = SynchronousIOMessage.serialize(response, io_buffer.data);
            }
            catch (error) {
                throw error;
            }
            finally {
                const semaphore = new Int32Array(io_buffer.semaphore);
                Atomics.store(semaphore, 0, signal);
                Atomics.notify(semaphore, 0);
            }
        }
        else {
            const { requestId } = event.data;
            const callbacks = this.requests.get(requestId);
            if (callbacks) {
                const { resolve, reject } = callbacks;
                if ('error' in event.data) {
                    reject(new Error(event.data.error));
                }
                else {
                    resolve(event.data.response);
                }
            }
        }
    }
}

// Copyright 2020 Google LLC. All rights reserved.
class WorkerPlugin {
    worker;
    rpc;
    constructor() {
        this.worker = new Worker('DevToolsPluginWorkerMain.bundle.js', { type: 'module' });
        this.rpc = new WorkerRPC(this.worker, this);
    }
    getWasmLinearMemory(offset, length, stopId) {
        return chrome.devtools.languageServices.getWasmLinearMemory(offset, length, stopId);
    }
    getWasmLocal(local, stopId) {
        return chrome.devtools.languageServices.getWasmLocal(local, stopId);
    }
    getWasmGlobal(global, stopId) {
        return chrome.devtools.languageServices.getWasmGlobal(global, stopId);
    }
    getWasmOp(op, stopId) {
        return chrome.devtools.languageServices.getWasmOp(op, stopId);
    }
    static async create(moduleConfigurations = DEFAULT_MODULE_CONFIGURATIONS, logPluginApiCalls = false) {
        const plugin = new WorkerPlugin();
        await plugin.rpc.sendMessage('hello', moduleConfigurations, logPluginApiCalls);
        return plugin;
    }
    addRawModule(rawModuleId, symbolsURL, rawModule) {
        return this.rpc.sendMessage('addRawModule', rawModuleId, symbolsURL, rawModule);
    }
    removeRawModule(rawModuleId) {
        return this.rpc.sendMessage('removeRawModule', rawModuleId);
    }
    sourceLocationToRawLocation(sourceLocation) {
        return this.rpc.sendMessage('sourceLocationToRawLocation', sourceLocation);
    }
    rawLocationToSourceLocation(rawLocation) {
        return this.rpc.sendMessage('rawLocationToSourceLocation', rawLocation);
    }
    getScopeInfo(type) {
        return this.rpc.sendMessage('getScopeInfo', type);
    }
    listVariablesInScope(rawLocation) {
        return this.rpc.sendMessage('listVariablesInScope', rawLocation);
    }
    getTypeInfo(expression, context) {
        return this.rpc.sendMessage('getTypeInfo', expression, context);
    }
    getFormatter(expressionOrField, context) {
        return this.rpc.sendMessage('getFormatter', expressionOrField, context);
    }
    getFunctionInfo(rawLocation) {
        return this.rpc.sendMessage('getFunctionInfo', rawLocation);
    }
    getInlinedFunctionRanges(rawLocation) {
        return this.rpc.sendMessage('getInlinedFunctionRanges', rawLocation);
    }
    getInlinedCalleesRanges(rawLocation) {
        return this.rpc.sendMessage('getInlinedCalleesRanges', rawLocation);
    }
    getMappedLines(rawModuleId, sourceFileURL) {
        return this.rpc.sendMessage('getMappedLines', rawModuleId, sourceFileURL);
    }
    getInspectableAddress(field) {
        return this.rpc.sendMessage('getInspectableAddress', field);
    }
    evaluate(expression, context, stopId) {
        return this.rpc.sendMessage('evaluate', expression, context, stopId);
    }
    getProperties(objectId) {
        return this.rpc.sendMessage('getProperties', objectId);
    }
    releaseObject(objectId) {
        return this.rpc.sendMessage('releaseObject', objectId);
    }
}
if (typeof chrome !== 'undefined' && typeof chrome.storage !== 'undefined') {
    const { storage } = chrome;
    const { languageServices } = chrome.devtools;
    async function registerPlugin(moduleConfigurations, logPluginApiCalls) {
        const plugin = await WorkerPlugin.create(moduleConfigurations, logPluginApiCalls);
        await languageServices.registerLanguageExtensionPlugin(plugin, 'C/C++ DevTools Support (DWARF)', { language: 'WebAssembly', symbol_types: ['EmbeddedDWARF', 'ExternalDWARF'] });
        return plugin;
    }
    async function unregisterPlugin(plugin) {
        await languageServices.unregisterLanguageExtensionPlugin(plugin);
    }
    const defaultConfig = {
        'moduleConfigurations': DEFAULT_MODULE_CONFIGURATIONS,
        'logPluginApiCalls': false,
    };
    chrome.storage.local.get(defaultConfig, ({ moduleConfigurations, logPluginApiCalls }) => {
        let pluginPromise = registerPlugin(moduleConfigurations, logPluginApiCalls);
        storage.onChanged.addListener(changes => {
            // Note that this doesn't use optional chaining '?.' as it is problematic in vscode.
            const moduleConfigurations = changes['moduleConfigurations'] !== undefined ? changes['moduleConfigurations'].newValue : undefined;
            const logPluginApiCalls = changes['logPluginApiCalls'] !== undefined ? changes['logPluginApiCalls'].newValue : undefined;
            if (moduleConfigurations || logPluginApiCalls !== undefined) {
                storage.local.get(defaultConfig, ({ moduleConfigurations, logPluginApiCalls }) => {
                    pluginPromise =
                        pluginPromise.then(unregisterPlugin).then(() => registerPlugin(moduleConfigurations, logPluginApiCalls));
                });
            }
        });
    });
}

export { WorkerPlugin };
