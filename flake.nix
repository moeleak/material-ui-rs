{
  description = "Material 3 inspired widgets for iced.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
        inherit (pkgs) lib;
        windowsPkgs = pkgs.pkgsCross.mingwW64;
        nativePackages = pkgs.callPackage ./package.nix { };
        windowsPackages = windowsPkgs.callPackage ./package.nix { };

        rustToolchain =
          with pkgs;
          [
            rustc
            rustfmt
            clippy
            rust-analyzer
            cargo
            curl
            lld
            trunk
            mdbook
            binaryen
            wasm-bindgen-cli
            nodejs_24
            python3
            python3Packages.fonttools
          ]
          ++ (lib.optionals pkgs.stdenv.isDarwin [ pkgs.libiconv ]);
      in
      {
        devShells.default = pkgs.mkShell {
          packages = rustToolchain ++ [ nativePackages.cargo_material_ui ];
          shellHook = ''
            export CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER="${pkgs.lld}/bin/wasm-ld"
          '';
        };

        packages = {
          inherit (nativePackages) default material_ui_rs cargo_material_ui;
          "material-ui-rs" = nativePackages.material_ui_rs;
          "cargo-material-ui" = nativePackages.cargo_material_ui;

          native = nativePackages.material_ui_rs;
          windows = windowsPackages.material_ui_rs;
          "material-ui-rs-windows" = windowsPackages.material_ui_rs;
        };
      }
    );
}
