{
  description = "confargs";

  inputs.fenix.url = github:nix-community/fenix;
  inputs.fenix.inputs.nixpkgs.follows = "nixpkgs";
  inputs.flake-utils.url = github:numtide/flake-utils;
  inputs.nixpkgs.url = github:NixOS/nixpkgs;

  outputs = {
    self,
    fenix,
    flake-utils,
    nixpkgs,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [fenix.overlay];
        };

        rust = with pkgs.fenix;
          combine [
            latest.toolchain
            targets.wasm32-wasi.latest.rust-std
            targets.x86_64-unknown-linux-gnu.latest.rust-std
            targets.x86_64-unknown-linux-musl.latest.rust-std
          ];
      in rec {
        formatter = pkgs.alejandra;

        devShells.default = pkgs.mkShell {
          buildInputs = [
            rust
          ];
        };
      }
    );
}
