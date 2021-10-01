{
  inputs = {
    nixpkgs.url = "nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, rust-overlay, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        devRustNightly = pkgs.rust-bin.nightly."2021-08-31".default.override {
          extensions = [ "rust-src" ];
          targets = [ "wasm32-unknown-unknown" ];
        };
        exampleTestScript = pkgs.writeShellScriptBin "run-example-test" ''
          cd examples/$1-contract
          cargo test
          rc=$?
          cd ../../
          exit $rc
        '';
        cliBuildTestScript = pkgs.writeShellScriptBin "cli-build-test" ''
          cd cargo-sewup
          cargo run -- -d -b -p ../examples/$1-contract
          cd ../
          ls -l examples/$1-contract/target/wasm32-unknown-unknown/release/$1_contract.deploy
          rc=$?
          exit $rc
        '';
        abiTestScript = pkgs.writeShellScriptBin "abi-test" ''
          cd cargo-sewup
          cargo run -- -g -p ../examples/$1-contract | jq --sort-keys > /tmp/$1_abi.json
          cd ../
          diff /tmp/$1_abi.json asset/$1_abi.json
          rc=$?
          exit $rc
        '';
        clientTestScript = pkgs.writeShellScriptBin "run-client-test" ''
          cd examples/$1-contract
          cargo build --bin $1-client --features=client
          rc=$?
          cd ../../
          exit $rc
        '';
        cliInitTestScript = pkgs.writeShellScriptBin "cli-init-test" ''
          cd cargo-sewup
          cargo run -- init -p /tmp/$1-proj -m $1
          cd /tmp/$1-proj
          cargo test
          rc=$?
          exit $rc
        '';
      in
      with pkgs;
      {
        devShell = mkShell {
          buildInputs = [
            boost
            clang
            cmake
            openssl
            pkg-config
            devRustNightly

            exampleTestScript
            cliBuildTestScript
            cliInitTestScript
            abiTestScript
            clientTestScript
          ];

          LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
          PROTOC = "${protobuf}/bin/protoc";
          ROCKSDB_LIB_DIR = "${rocksdb}/lib";
        };
      }
    );
}
