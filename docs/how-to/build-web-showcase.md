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

## Mobile IME Bridge

The showcase host page includes a hidden-input bridge for Android and iOS
browsers. It focuses a real DOM input from the trusted touch gesture, keeps IME
composition keys inside the browser, and forwards committed text plus
`beforeinput` delete and Enter actions to the iced canvas.

The CJK font is also demand-loaded: the initial WASM page does not request it
until CJK text is first entered in the note field.

Run its DOM-level regression tests with Node.js 24 or newer:

```sh
node --test web/mobile_ime.test.mjs
```

Applications using their own HTML shell need equivalent JavaScript hooks for
`__icedMaterialShowMobileKeyboard`, `__icedMaterialHideMobileKeyboard`, and
`__icedMaterialRegisterTextRegion`; use `web/index.html` as the integration
template.
