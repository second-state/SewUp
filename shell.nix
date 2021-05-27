let
  mozillaOverlay =
    import (builtins.fetchGit {
      url = "https://github.com/mozilla/nixpkgs-mozilla.git";
      rev = "57c8084c7ef41366993909c20491e359bbb90f54";
    });
  nixpkgs = import <nixpkgs> { overlays = [ mozillaOverlay ]; };
  rust = with nixpkgs; ((rustChannelOf { date = "2021-05-06"; channel = "stable"; }).rust.override {
    targets = [ "wasm32-unknown-unknown" ];
  });

  updateConteact = nixpkgs.writeShellScriptBin "update-contract" ''
    cd erc20-contract
    cargo build --release
    cd ../
    mv target/wasm32-unknown-unknown/release/erc20_contract.wasm resources/test/erc20_contract.wasm
  '';
  testScript = nixpkgs.writeShellScriptBin "run-test" ''
    cargo test -p sewup --features=$1 -- --nocapture | tee /tmp/vm_errors && exit $(grep ERROR /tmp/vm_errors | wc -l)
  '';
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

    testScript
    updateConteact
  ] ++ lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.Security
  ];
  nativeBuildInputs = with nixpkgs; [
    cmake
  ];
  LIBCLANG_PATH = "${nixpkgs.llvmPackages_10.libclang.lib}/lib";
}
