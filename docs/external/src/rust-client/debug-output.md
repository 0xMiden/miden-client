---
title: MASM Debug Output
sidebar_position: 8
---

# MASM Debug Output

Miden Assembly's `debug.*` instructions (`debug.stack`, `debug.stack.<n>`, `debug.mem`,
`debug.local.<a>.<b>`, `debug.adv_stack.<n>` — see the
[assembly reference](https://github.com/0xMiden/miden-vm/blob/next/docs/src/user_docs/assembly/debugging.md))
print VM state to the client's standard output while a script runs. This is a lightweight
alternative to [interactive DAP debugging](./debugging.md).

They only print when the client that **executes** the script is in debug mode — enable it with
`.in_debug_mode(DebugMode::Enabled)` on the builder, or run the CLI with `--debug` (or
`MIDEN_DEBUG=true`). Compilation is unaffected; the decorators are always retained in freshly
compiled scripts. Output goes to the client's standard output (not `tracing`/`RUST_LOG`, not the
node logs), for example:

```text
Stack state before step 5:
├── 0: 1
├── 1: 2
└── ...
```

:::note
Under tests, pass `--no-capture` (`cargo nextest`, used by `make test`) or `--nocapture`
(`cargo test`) to see the output.
:::
