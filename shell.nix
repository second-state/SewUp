let
  mozillaOverlay =
    import (builtins.fetchGit {
      url = "https://github.com/mozilla/nixpkgs-mozilla.git";
      rev = "57c8084c7ef41366993909c20491e359bbb90f54";
    });
  nixpkgs = import <nixpkgs> { overlays = [ mozillaOverlay ]; };
  rust = with nixpkgs; ((rustChannelOf { date = "2021-05-06"; channel = "stable"; }).rust.override {
    extensions = [ "rust-src" ];
  });

  clangStdenv = nixpkgs.llvmPackages_10.stdenv;
in
clangStdenv.mkDerivation {
  name = "clang-10-nix-shell";
  buildInputs = with nixpkgs; [
    cmake
    pkg-config
    rust
    clippy

    llvmPackages_10.llvm
    lld_10
    boost
  ] ++ stdenv.lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.Security
  ];
}
