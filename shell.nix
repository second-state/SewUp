let
  mozillaOverlay =
    import (builtins.fetchGit {
      url = "https://github.com/mozilla/nixpkgs-mozilla.git";
      rev = "57c8084c7ef41366993909c20491e359bbb90f54";
    });
  nixpkgs = import <nixpkgs> { overlays = [ mozillaOverlay ]; };
  rust-stable = with nixpkgs; ((rustChannelOf { date = "2021-05-06"; channel = "stable"; }).rust.override {
    targets = [ "wasm32-unknown-unknown" ];
  });
  updateContract = nixpkgs.writeShellScriptBin "update-contract" ''
    cd erc20-contract
    cargo build --release
    cd ../
    mv target/wasm32-unknown-unknown/release/erc20_contract.wasm resources/test/erc20_contract.wasm
  '';
  testScript = nixpkgs.writeShellScriptBin "run-test" ''
    cd sewup
    cargo test -p sewup --features=$1 -- --nocapture | tee /tmp/vm_errors && exit $(grep ERROR /tmp/vm_errors | wc -l)
    cd ../
  '';
in
with nixpkgs; pkgs.mkShell {
  buildInputs = [
    clang
    cmake
    pkg-config
    rust-stable
    boost
    updateContract
    testScript
  ] ++ lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.Security
  ];

  LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
  PROTOC = "${protobuf}/bin/protoc";
  ROCKSDB_LIB_DIR = "${rocksdb}/lib";
}
