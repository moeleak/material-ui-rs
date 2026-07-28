{
  description = "Generated material-ui-rs development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, flake-utils, rust-overlay, ... }:
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
{{platform_packages}}{{android_package}}          ];

          shellHook = ''
{{android_hook}}{{web_hook}}          '';
        };
      });
}
