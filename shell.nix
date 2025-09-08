with import <nixpkgs> { };

mkShell {
  packages = [ rustc cargo gcc rustfmt clippy mdbook just ];
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
