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
            rust-analyzer
            cargo
            lld
            trunk
            mdbook
            binaryen
            wasm-bindgen-cli
            nodejs_24
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

        packages = {
          inherit (nativePackages) default iced_material;

          native = nativePackages.iced_material;
          windows = windowsPackages.iced_material;
          iced_material-windows = windowsPackages.iced_material;
        };
      }
    );
}
