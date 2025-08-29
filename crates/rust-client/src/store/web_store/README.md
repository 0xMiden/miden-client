## Formatting for .js files
Please install the VSCode prettier extension and set as default formatter for js 

## Javascript & Typescript interop

Besides from compiling Rust to WebAssembly, we sometimes have the need to call some external
Javascript function. This is not bad per se, but it has a problem: Javascript is a
dynamic, weakly typed language, any type guarantees we get from Rust are erased once we start 
calling JS code. To mitigate that, we've started to incorporate Typescript where we once used 
Javascript. The setup consists of .ts files under the `web_store/ts` that get compiled to .js 
files under `web_store/js`. This is because to use extern functions, we still need to import raw
.js files.

To unify and make this setup straightforward, the top-most makefile from this project has a
useful target: `make rust-client-ts-build`, which takes the .ts files and compiles them down to .js files.

