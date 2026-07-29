{
  lib,
  stdenv,
  buildPackages,
  rustPlatform,
  libiconv,
  writableTmpDirAsHomeHook,
}:
let
  version = "0.5.1";
  src = lib.cleanSource ./.;
  cargoMaterialUiSrc = lib.cleanSource ./tools/cargo-material-ui;
  executableSuffix = stdenv.hostPlatform.extensions.executable or "";
  darwinHostLinkAttrs = lib.optionalAttrs stdenv.buildPlatform.isDarwin {
    RUSTFLAGS = "-L native=${buildPackages.libiconv}/lib";
    env.LIBRARY_PATH = "${buildPackages.libiconv}/lib";
  };
in
rec {
  default = material_ui_rs;

  cargo_material_ui = rustPlatform.buildRustPackage {
    pname = "cargo-material-ui";
    version = "0.1.0";
    src = cargoMaterialUiSrc;

    cargoLock = {
      lockFile = ./tools/cargo-material-ui/Cargo.lock;
    };

    doCheck = stdenv.buildPlatform.canExecute stdenv.hostPlatform;

    installPhase = ''
      runHook preInstall

      binary="$(find target -type f -path "*/release/cargo-material-ui${executableSuffix}" | head -n 1)"

      if [ -z "$binary" ]; then
        echo "cargo-material-ui binary not found" >&2
        find target -maxdepth 5 -type f >&2
        exit 1
      fi

      install -Dm755 "$binary" "$out/bin/cargo-material-ui${executableSuffix}"

      runHook postInstall
    '';

    meta = {
      description = "Create and build multi-platform material-ui-rs applications";
      license = lib.licenses.mit;
      mainProgram = "cargo-material-ui";
    };
  };

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
