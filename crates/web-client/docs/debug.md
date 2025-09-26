# Debugging WASM

When debugging WASM, it can be hard to trace the origin of an error since stack traces are not really useful.
The typical situation is that you're running a frontend app from JS and get an unreadeable stacktrace.
We can generate a debug build with symbols to help us trace these errors, this document is the setup for it.


## Requirements
    - yarn
    - rust
    - Chrome browser
    - You will need to install this [extension](https://goo.gle/wasm-debugging-extension).
      Despite the name, it will also work with WASM-generated Rust Files
    
## Building with debug symbols

1. cd into to the root of the web-client project, that is: `miden-client/crates/web-client`.
2. build the dev profile with: 
   ```yarn build-dev```
3. Once it finishes, the rust build log should print this:
  ```
    Finished `dev` profile [optimized + debuginfo] target(s) in 38.33s
  ```
  4. (Optional), to double check the debug symbols have been generated, you can install
  the Web Assembly Binary Toolkit. A few sources are:
  - [brew package manger](https://formulae.brew.sh/formula/wabt)
  
  - [nix packages](https://github.com/NixOS/nixpkgs/blob/25e53aa156d47bad5082ff7618f5feb1f5e02d01/pkgs/by-name/wa/wabt/package.nix#L27)
 - [ubuntu](https://launchpad.net/ubuntu/+source/wabt)  (not tested)
 - [source](https://github.com/WebAssembly/wabt).
 The wabt package provides an `wasm-obj` binary, which you can use like so:
 ```
 wasm-objdump --headers dist/workers/assets/miden_client_web.wasm
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

Once you have both the debug wasm and the chrome extension, we need to link
the dependency to the JS app we're debugging .

1. Link the package, cd to: `miden-client/crates/web-client and run:
```
yarn link 
```
2. In the root of the project you're debugging, run this:
```
yarn link "@demox-labs/miden-sdk"
```
3. When you open the devtools with chrome, you will see an output like this:
![dev-tools-output](./devtools-output.png).

You should also be able to see the rust source in the devtools source tab:
![source-example](./source-example.png)

Also, you should seed friendlier stack-traces:
![stack-trace-example](./stack-trace-example.png)


## Relevant changes

This changes are already reflected in the codebase, but the settings to make the dev build have debug symbols are the following:

1. The root Cargo.toml needs to optimize the dev profile for size, otherwise it wont compile:
```
[profile.dev]
opt-level = "s"
```
2. In the rollup.config.js the relevante options for the rust plugin are:
```
{
   extraArgs: {
       wasmBindgen: [ "--keep-debug"],
   } 
   optimize: {
       release: false,
       rustc: false
   }
}
```
