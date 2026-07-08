# Build the WebAssembly Showcase

The WebAssembly showcase uses Trunk and the `showcase_web` binary.

From the repository root:

```sh
trunk build web/index.html --release --dist dist --public-url /
python3 -m http.server 4173 --directory dist
```

Open:

```text
http://127.0.0.1:4173/
```

Serve `dist/` over HTTP instead of opening `dist/index.html` directly. Browsers
need the server to return the JavaScript module and WASM file with the correct
MIME types.

With Nix:

```sh
nix develop -c trunk build web/index.html --release --dist dist --public-url /
nix develop -c python3 -m http.server 4173 --directory dist
```

The `dist/` directory is ignored by the repository.
