// world root:component/root
import type * as ArboriumHostPluginProvider from './interfaces/arborium-host-plugin-provider.js'; // arborium:host/plugin-provider@0.1.0
import type * as ArboriumHostTypes from './interfaces/arborium-host-types.js'; // arborium:host/types@0.1.0
import type * as WasiCliEnvironment from './interfaces/wasi-cli-environment.js'; // wasi:cli/environment@0.2.3
import type * as WasiCliExit from './interfaces/wasi-cli-exit.js'; // wasi:cli/exit@0.2.3
import type * as WasiCliStderr from './interfaces/wasi-cli-stderr.js'; // wasi:cli/stderr@0.2.3
import type * as WasiCliStdin from './interfaces/wasi-cli-stdin.js'; // wasi:cli/stdin@0.2.3
import type * as WasiCliStdout from './interfaces/wasi-cli-stdout.js'; // wasi:cli/stdout@0.2.3
import type * as WasiClocksWallClock from './interfaces/wasi-clocks-wall-clock.js'; // wasi:clocks/wall-clock@0.2.3
import type * as WasiFilesystemPreopens from './interfaces/wasi-filesystem-preopens.js'; // wasi:filesystem/preopens@0.2.3
import type * as WasiFilesystemTypes from './interfaces/wasi-filesystem-types.js'; // wasi:filesystem/types@0.2.3
import type * as WasiIoError from './interfaces/wasi-io-error.js'; // wasi:io/error@0.2.3
import type * as WasiIoStreams from './interfaces/wasi-io-streams.js'; // wasi:io/streams@0.2.3
import type * as ArboriumHostHost from './interfaces/arborium-host-host.js'; // arborium:host/host@0.1.0
export interface ImportObject {
  'arborium:host/plugin-provider@0.1.0': typeof ArboriumHostPluginProvider,
  'arborium:host/types@0.1.0': typeof ArboriumHostTypes,
  'wasi:cli/environment@0.2.3': typeof WasiCliEnvironment,
  'wasi:cli/exit@0.2.3': typeof WasiCliExit,
  'wasi:cli/stderr@0.2.3': typeof WasiCliStderr,
  'wasi:cli/stdin@0.2.3': typeof WasiCliStdin,
  'wasi:cli/stdout@0.2.3': typeof WasiCliStdout,
  'wasi:clocks/wall-clock@0.2.3': typeof WasiClocksWallClock,
  'wasi:filesystem/preopens@0.2.3': typeof WasiFilesystemPreopens,
  'wasi:filesystem/types@0.2.3': typeof WasiFilesystemTypes,
  'wasi:io/error@0.2.3': typeof WasiIoError,
  'wasi:io/streams@0.2.3': typeof WasiIoStreams,
}
export interface Root {
  'arborium:host/host@0.1.0': typeof ArboriumHostHost,
  host: typeof ArboriumHostHost,
}

/**
* Instantiates this component with the provided imports and
* returns a map of all the exports of the component.
*
* This function is intended to be similar to the
* `WebAssembly.instantiate` function. The second `imports`
* argument is the "import object" for wasm, except here it
* uses component-model-layer types instead of core wasm
* integers/numbers/etc.
*
* The first argument to this function, `getCoreModule`, is
* used to compile core wasm modules within the component.
* Components are composed of core wasm modules and this callback
* will be invoked per core wasm module. The caller of this
* function is responsible for reading the core wasm module
* identified by `path` and returning its compiled
* `WebAssembly.Module` object. This would use `compileStreaming`
* on the web, for example.
*/
export function instantiate(
getCoreModule: (path: string) => WebAssembly.Module,
imports: ImportObject,
instantiateCore?: (module: WebAssembly.Module, imports: Record<string, any>) => WebAssembly.Instance
): Root;
export function instantiate(
getCoreModule: (path: string) => WebAssembly.Module | Promise<WebAssembly.Module>,
imports: ImportObject,
instantiateCore?: (module: WebAssembly.Module, imports: Record<string, any>) => WebAssembly.Instance | Promise<WebAssembly.Instance>
): Root | Promise<Root>;

