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
  exampleTestScript = nixpkgs.writeShellScriptBin "run-example-test" ''
    cd examples/$1-contract
    cargo test
    cd ../../
  '';
in
with nixpkgs; pkgs.mkShell {
  buildInputs = [
    clang
    cmake
    pkg-config
    rust-stable
    boost
    exampleTestScript
  ] ++ lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.Security
  ];

  LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
  PROTOC = "${protobuf}/bin/protoc";
  ROCKSDB_LIB_DIR = "${rocksdb}/lib";
}
