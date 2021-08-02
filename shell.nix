let
  mozillaOverlay =
    import (builtins.fetchGit {
      url = "https://github.com/mozilla/nixpkgs-mozilla.git";
      rev = "57c8084c7ef41366993909c20491e359bbb90f54";
    });
  nixpkgs = import <nixpkgs> { overlays = [ mozillaOverlay ]; };
  rust-nightly = with nixpkgs; ((rustChannelOf { date = "2021-05-14"; channel = "nightly"; }).rust.override {
    targets = [ "wasm32-unknown-unknown" ];
  });
  exampleTestScript = nixpkgs.writeShellScriptBin "run-example-test" ''
    cd examples/$1-contract
    cargo test
    rc=$?
    cd ../../
    exit $rc
  '';
  cliTestScript = nixpkgs.writeShellScriptBin "cli-test" ''
    cd cargo-sewup
    cargo run -- -v -d -b -p ../examples/default-contract
    cd ../
    diff examples/default-contract/target/wasm32-unknown-unknown/release/default_contract.deploy \
      resources/test/default_contract.deploy
    rc=$?
    exit $rc
  '';
in
with nixpkgs; pkgs.mkShell {
  buildInputs = [
    boost
    clang
    cmake
    openssl
    pkg-config
    rust-nightly

    exampleTestScript
    cliTestScript
  ] ++ lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.Security
  ];

  LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
  PROTOC = "${protobuf}/bin/protoc";
  ROCKSDB_LIB_DIR = "${rocksdb}/lib";
}
