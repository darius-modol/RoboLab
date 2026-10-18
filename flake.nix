# If you don't know what this file is, this is not relevant to you.
# It won't do anything at all and you can safely ignore it.
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs =
    { flake-utils
    , nixpkgs
    , rust-overlay
    , ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
      in
      {
        devShells.default = with pkgs; mkShell rec {
          buildInputs = [
          ];
          nativeBuildInputs = [
            (rust-bin.stable.latest.default.override {
              targets = [ "armv5te-unknown-linux-musleabi" "aarch64-unknown-linux-musl" ];
            })
            pkg-config
            cargo-nextest
            cargo-zigbuild
            zig

            treefmt
            nixfmt
          ];
          LD_LIBRARY_PATH = (nixpkgs.lib.makeLibraryPath buildInputs);
          shellHook = ''
            export PATH=${pkgs.rust-analyzer}/bin:$PATH
            export RUST_LOG=info
          '';
        };
      }
    );
}
