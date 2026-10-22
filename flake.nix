{
  description = "Postal Converter JA Development Environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    # Security backport includes Node.js 24.22.1 and shared Brotli 1.2.1.
    nodejs-nixpkgs.url = "github:NixOS/nixpkgs/04b65f5f819c8f0d0592a4b34fb3ee57ac92b1ad";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, nodejs-nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        nodejsPkgs = nodejs-nixpkgs.legacyPackages.${system};
        nodejs =
          assert nodejsPkgs.lib.versionAtLeast nodejsPkgs.nodejs_24.version "24.22.1";
          assert nodejsPkgs.lib.versionAtLeast nodejsPkgs.brotli.version "1.2.1";
          nodejsPkgs.nodejs_24;
        # Rust toolchain
        rustToolchain = pkgs.rust-bin.stable."1.97.1".default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        # Keep local command UX as `terraform` while using OSS OpenTofu in nix shell.
        terraformCompat = pkgs.writeShellScriptBin "terraform" ''
          exec ${pkgs.opentofu}/bin/tofu "$@"
        '';

        # System dependencies
        buildInputs = with pkgs; [
          nodejsPkgs.openssl
          pkg-config
          curl
          wget
          go
          nodejs
          (yarn.override { inherit nodejs; })
          sqlite
          opentofu
          terraformCompat
        ] ++ pkgs.lib.optionals pkgs.stdenv.hostPlatform.isDarwin [
          pkgs.libiconv
        ];

      in
      {
        packages.nodejs = nodejs;
        checks.nodejs-security = nodejsPkgs.runCommand "nodejs-security" {
          nativeBuildInputs = [ nodejs ];
          # Exercise the same shared-library search path as the development shell.
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
        } ''
          node <<'JS'
          const assert = require("node:assert/strict");
          const zlib = require("node:zlib");
          assert.equal(process.versions.node, "${nodejs.version}");
          assert.equal(process.versions.brotli, "${nodejsPkgs.brotli.version}");
          assert.equal(process.versions.openssl, "${nodejsPkgs.openssl.version}");
          const input = Buffer.from("Brotli runtime check");
          assert.deepEqual(zlib.brotliDecompressSync(zlib.brotliCompressSync(input)), input);
          console.log(process.versions.node, process.versions.brotli, process.versions.openssl);
          JS
          touch "$out"
        '';

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
