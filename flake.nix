{
  description = "rust flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix/monthly";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.fenix.follows = "fenix";
    };
  };

  outputs = { self, nixpkgs, flake-utils, fenix, naersk }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ fenix.overlays.default ];
        };
        rust-toolchain = pkgs.fenix.stable.toolchain;
        deps = with pkgs; [
          webkitgtk_6_0
          gtk4
          libsoup_3
          gdk-pixbuf
          cairo
          glib
        ];
      in
      {
        devShells.default =
        pkgs.mkShell {
          buildInputs = deps;

          nativeBuildInputs = with pkgs; [
            rust-toolchain
            rust-analyzer

            # extra cargo tools
            cargo-edit
            cargo-expand

            # pkg config
            pkg-config
          ];
          
          # set library path, as rpath can't be set when building manually
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath deps;
          # set the rust src for rust_analyzer
          RUST_SRC_PATH = "${rust-toolchain}/lib/rustlib/src/rust/library";
        };

        packages.default = 
        (naersk.lib.${system}.override {
          cargo = rust-toolchain;
          rustc = rust-toolchain;
        }).buildPackage {
          src = ./.;
          buildInputs = deps;
          nativeBuildInputs = [ pkgs.pkg-config pkgs.autoPatchelfHook ];
        };
      }
    );
}
