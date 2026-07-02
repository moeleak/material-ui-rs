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

        rustPlatform = pkgs.makeRustPlatform {
          cargo = pkgs.cargo;
          rustc = pkgs.rustc;
        };

        rustToolchain =
          with pkgs;
          [
            rustc
            rustfmt
            rust-analyzer
            cargo
            lld
            trunk
            binaryen
            wasm-bindgen-cli
          ]
          ++ (lib.optionals pkgs.stdenv.isDarwin [ pkgs.libiconv ]);
      in
      {
        devShells.default = pkgs.mkShell {
          packages = rustToolchain;
          shellHook = ''
            export CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER="${pkgs.lld}/bin/wasm-ld"
          '';
        };

        packages.default = rustPlatform.buildRustPackage {
          pname = "iced_material";
          version = "0.14.2";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          buildInputs = lib.optionals pkgs.stdenv.isDarwin [ pkgs.libiconv ];
        };
      }
    );
}
