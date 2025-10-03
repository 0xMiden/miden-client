# Debugging WASM

When debugging WASM, it can be tricky to trace the origin of an error since stack traces are not often noisy and difficult to read.
To better trace these errors, a build with debug symbols can be generated. This doc explains how to set it up.

## Requirements
    - yarn
    - Compatible Rust version
    - Chrome browser
    - [wasm-debugging-extension](https://goo.gle/wasm-debugging-extension).
    
## Building with debug symbols

1. Clone the miden-client repo:
```bash
git clone git@github.com:0xMiden/miden-client.git
```
2. Build miden-client with debug-symbols:
```bash
make build-web-client-debug
```
3. Once it finishes, the Rust build log should print this:
  ```
    Finished `release` profile [optimized + debuginfo] target(s) in 38.33s
  ```
  4. (Optional) To double-check that debug symbols were generated, install the WebAssembly Binary Toolkit (WABT). Sources:
  - [brew package manager](https://formulae.brew.sh/formula/wabt)
  - [nix packages](https://github.com/NixOS/nixpkgs/blob/25e53aa156d47bad5082ff7618f5feb1f5e02d01/pkgs/by-name/wa/wabt/package.nix#L27)
  - [source](https://github.com/WebAssembly/wabt).

 The WABT package provides an `wasm-obj` binary, which you can use like so:
 ```
 wasm-objdump --headers crates/web-client/dist/workers/assets/miden_client_web.wasm
 ```
 If the debug symbols are present, you should see a bunch of "debug" headers.
 An example of an output like this would be:
 ```
   Custom start=0x00a85ee7 end=0x00a85f63 (size=0x0000007c) "producers"
   Custom start=0x00a85f67 end=0x00bc6700 (size=0x00140799) ".debug_abbrev"
   Custom start=0x00bc6705 end=0x03f1989b (size=0x03353196) ".debug_str"
   Custom start=0x03f198a0 end=0x0480968b (size=0x008efdeb) ".debug_line"
   Custom start=0x04809690 end=0x04c347ae (size=0x0042b11e) ".debug_ranges"
   Custom start=0x04c347b3 end=0x059ae810 (size=0x00d7a05d) ".debug_loc"
   Custom start=0x059ae815 end=0x09113060 (size=0x0376484b) ".debug_info"
 ```
    
## Using the debug symbols

Once you have both the debug WASM and the Chrome extension, we need to link
the dependency to the JS app we're debugging.

1. Once you have the web client built with debug symbols, we have to use it as a dependency,
for that we'll use [yarn link](https://classic.yarnpkg.com/lang/en/docs/cli/link/), 
run this in your local copy of miden-client:
```
make link-web-client-dep
```
2. Then, run this command in the root of the project that needs to be debugged:
```
yarn link "@demox-labs/miden-sdk"
```
Essentially,`yarn link` makes the project use the local modified version of the sdk instead of the NPM hosted one. Now, when you open the devtools with chrome, you will see an output like this:


![dev-tools-output](./devtools-output.png).


You should also be able to see the rust source in the devtools source tab:
![source-example](./source-example.png)


Also, you should see friendlier stack-traces:
![stack-trace-example](./stack-trace-example.png)


## Relevant changes

These changes are already reflected in the codebase, but the settings to make the release build have debug symbols are the following:

1. In the `rollup.config.js` the relevant options for the Rust plugin are:
```
{
   extraArgs: {
     <other args>,
     wasmBindgen: [ "--keep-debug"],
   } 
}
```
2. The rollup plugin also calls cargo internally, which needs these arguments to add debug symbols:
   
```javascript
const cargoArgsUseDebugSymbols = [
  // Generate debug symbols for the release cargo profile.
  "--config",
  "profile.release.debug='full'",
  // Do not remove debug symbols from the final binary,
  "--config",
  "profile.release.strip='none'",
];
```
