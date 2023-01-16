{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    # Dependencies
    cargo
    clippy
    rustc
    rustfmt
    rust-analyzer
  ];

  RUST_SRC_DIR = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
