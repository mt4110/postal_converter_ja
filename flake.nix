{
  description = "Postal Converter JA Development Environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        # Rust toolchain
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        # System dependencies
        buildInputs = with pkgs; [
          openssl
          pkg-config
          curl
          wget
          go
        ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
          pkgs.libiconv
          pkgs.darwin.apple_sdk.frameworks.Security
          pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
        ];

      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            rustToolchain
          ] ++ buildInputs;

          shellHook = ''
            export RUST_SRC_PATH="${rustToolchain}/lib/rustlib/src/rust/library"
            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath buildInputs}:$LD_LIBRARY_PATH"
            echo "Welcome to Postal Converter JA Dev Shell!"
            echo "Rust version: $(rustc --version)"
          '';
        };
      }
    );
}
