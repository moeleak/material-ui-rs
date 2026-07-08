{
  lib,
  stdenv,
  buildPackages,
  rustPlatform,
  libiconv,
  writableTmpDirAsHomeHook,
}:
let
  version = "0.4.0";
  src = lib.cleanSource ./.;
  executableSuffix = stdenv.hostPlatform.extensions.executable or "";
  darwinHostLinkAttrs = lib.optionalAttrs stdenv.buildPlatform.isDarwin {
    RUSTFLAGS = "-L native=${buildPackages.libiconv}/lib";
    env.LIBRARY_PATH = "${buildPackages.libiconv}/lib";
  };
in
rec {
  default = material_ui_rs;

  material_ui_rs = rustPlatform.buildRustPackage ({
    inherit version src;

    pname = "material-ui-rs";

    cargoLock = {
      lockFile = ./Cargo.lock;
    };

    cargoBuildFlags = [
      "--example"
      "showcase"
    ];

    cargoTestFlags = [
      "--lib"
      "--tests"
    ];

    doCheck = stdenv.buildPlatform.canExecute stdenv.hostPlatform;

    nativeBuildInputs =
      lib.optionals stdenv.buildPlatform.isDarwin [
        libiconv
      ]
      ++ lib.optionals (stdenv.buildPlatform.canExecute stdenv.hostPlatform) [
        writableTmpDirAsHomeHook
      ];

    installPhase = ''
      runHook preInstall

      binary="$(find target -type f -path "*/release/examples/showcase${executableSuffix}" | head -n 1)"

      if [ -z "$binary" ]; then
        echo "showcase binary not found" >&2
        find target -maxdepth 5 -type f >&2
        exit 1
      fi

      install -Dm755 "$binary" "$out/bin/material-ui-rs${executableSuffix}"

      runHook postInstall
    '';

    meta = {
      license = lib.licenses.mit;
      mainProgram = "material-ui-rs";
    };
  } // darwinHostLinkAttrs);
}
