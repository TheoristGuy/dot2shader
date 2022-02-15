# dot2shader-gui

Simple GUI wrapping `dot2shader`.  
Pre-build page is [here](https://iwbtshyguy.gitlab.io/dot2shader/).

## Run native

This application can be run both natively and on the web. To run it natively is easy, just type the following command on this directory.

```bash
cargo run
```

## Build for the web

### Prepare

```bash
# Build tool for Rust/wasm. The latest version due to open-ssl errors,
cargo install wasm-pack --version 0.9.1
# Useful for running wasm. http-server from npm cannot set the MIME type application/wasm (as far as I know).
cargo install basic-http-server
```

### Run on the web

```bash
# If this is not enabled, the generated source code cannot be copied and pasted.
export RUSTFLAGS=--cfg=web_sys_unstable_apis
# build by wasm-pack
wasm-pack build --target web
# copy HTML and icon into target file.
cp resources/index.html resources/favicon.ico pkg
# Start up a server
basic-http-server pkg
```

Enter the url `localhost:4000` to the browser!
