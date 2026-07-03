# Building the WASM demo

This demo runs `spreadsheet_lib` (the same engine the CLI uses) compiled to
WebAssembly, driven by `wasm_app` (a thin `wasm-bindgen` wrapper) and a plain
HTML/JS grid UI in this folder.

## One-time setup (skip anything you already have)

```bash
# Rust's WASM compile target
rustup target add wasm32-unknown-unknown

# wasm-bindgen CLI + build orchestration, via cargo
cargo install wasm-pack
```

## Build

From the repo root:

```bash
cd wasm_app
wasm-pack build --target web --out-dir ../demo/pkg
```

This compiles `wasm_app` (and its `spreadsheet_lib` dependency) to
`wasm32-unknown-unknown`, runs `wasm-bindgen` over the output, and drops
`wasm_app.js` + `wasm_app_bg.wasm` (+ types) into `demo/pkg/`.

## Run it

WASM + ES modules need to be served over HTTP (not opened as a `file://`
URL, browsers block module imports from disk). From the repo root:

```bash
cd demo
python3 -m http.server 8080
```

Then open `http://localhost:8080` in a browser.

## If the build fails

Most likely cause: the `wasm-bindgen` crate version pinned in
`wasm_app/Cargo.toml` (`0.2`) doesn't match the `wasm-bindgen-cli` version
`wasm-pack` picked. If you see an error like "it looks like the
Rust project used to create this wasm file was linked against version of
wasm-bindgen that is different than this binary", run:

```bash
cargo update -p wasm-bindgen
```

and rebuild. Paste me the exact error if it's something else and I'll fix it.

## Deploying

Once `demo/pkg/` exists, the whole `demo/` folder is a static site --
GitHub Pages can serve it directly by pointing at `/demo` on the `main`
branch (Settings -> Pages -> Deploy from a branch -> `/demo`).
