{
  description = "Generated material-ui-rs development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    material-ui-rs = {
      url = "github:moeleak/material-ui-rs";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
  };

  outputs = { nixpkgs, flake-utils, rust-overlay, material-ui-rs, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
          config = {
            allowUnfree = true;
            android_sdk.accept_license = true;
          };
        };
        materialUiCli = material-ui-rs.packages.${system}.cargo-material-ui;
        rustToolchain = pkgs.rust-bin.stable."1.88.0".default.override {
          targets = [
{{rust_targets}}          ];
        };
{{android_binding}}
      in {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            rustToolchain
            rust-analyzer
            lld
            materialUiCli
{{platform_packages}}{{android_package}}          ];

          shellHook = ''
{{android_hook}}{{web_hook}}          '';
        };
      });
}
