<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <meta name="color-scheme" content="dark light" />
    <base data-trunk-public-url />
    <title>{{label}}</title>
    <style>
      html,
      body {
        width: 100%;
        height: 100%;
        margin: 0;
      }

      body {
        overflow: hidden;
        background: #141218;
      }

      canvas {
        display: block;
      }
    </style>
    <link
      data-trunk
      rel="rust"
      href="../Cargo.toml"
      data-bin="{{package_name}}"
      data-wasm-opt="z"
    />
  </head>
  <body></body>
</html>
