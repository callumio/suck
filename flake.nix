{
  description = "`suck` development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";

      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      overlays = [(import rust-overlay)];
      pkgs = import nixpkgs {inherit system overlays;};
      rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      rustToolchainNightly = pkgs.pkgsBuildHost.rust-bin.nightly.latest.default;
      tools = with pkgs; [cargo-nextest];
      nativeBuildInputs = with pkgs; [rustToolchain rustToolchainNightly pkg-config] ++ tools;
    in
      with pkgs; {
        devShells.default = mkShell {
          inherit nativeBuildInputs;
          shellHook = ''
            export CARGO_NIGHTLY="${rustToolchainNightly}/bin/cargo"
          '';
        };
      });
}
