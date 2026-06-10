# Web (WebAssembly) build

The release workflow builds `wolf3d-rs` for `wasm32-unknown-unknown` and packages
it together with the files in this directory into a `…-web.tar.gz` artifact.

## Running locally

```sh
cd rust
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown

# Lay the files out the way the release artifact does:
mkdir -p web-dist
cp target/wasm32-unknown-unknown/release/wolf3d-rs.wasm web-dist/
cp web/index.html web/mq_js_bundle.js web-dist/

# Serve over HTTP (WASM cannot be loaded from file://):
python3 -m http.server -d web-dist 8080
# open http://localhost:8080
```

## `mq_js_bundle.js` provenance

`mq_js_bundle.js` is the miniquad/macroquad JavaScript glue, vendored here so the
web build has **no live CDN dependency** (a remote `<script src>` would be a
supply-chain risk and break offline use).

- Source: <https://not-fl3.github.io/miniquad-samples/mq_js_bundle.js>
- SHA-256: `f4189c03de0b10cef3b710bdcadb22df7b835ccab5de52a3714e896e4fc48cd7`

To update it, re-download from the source above, verify the new checksum, and
record it here in the same commit.
