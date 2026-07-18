{
  description = "A flake.nix to allow easy development with the Orichalcum framework for Nix aficionados.";
  inputs = {
    cargo2nix.url = "github:cargo2nix/cargo2nix/release-0.12";
    flake-utils.follows = "cargo2nix/flake-utils";
    nixpkgs.follows = "cargo2nix/nixpkgs";
  };

  outputs = inputs: with inputs;
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [cargo2nix.overlays.default];
        };

        rustPkgs = pkgs.rustBuilder.makePackageSet {
          rustVersion = "1.85.0";
          packageFun = import ./Cargo.nix;
        };
        rustToolchain = pkgs.rust-bin.stable."1.85.0".default.override {
          extensions = [ "clippy" "rustfmt" "rust-src" ];
        };

      in rec {
        # This section defines the "finished product"
        packages = {
          orichalcum = (rustPkgs.workspace.orichalcum {});
          default = packages.orichalcum;
        };

        # This section defines the "developer workshop"
        devShells.default = pkgs.mkShell {
          # Pulls in all build dependencies (rustc, cargo)
          # from 'orichalcum' package.
          inputsFrom = [ packages.orichalcum ];

          nativeBuildInputs = [
            rustToolchain
            pkgs.rust-analyzer
            pkgs.cargo-edit
            pkgs.pkg-config
            pkgs.openssl
          ];
        };
      }
    );
}
