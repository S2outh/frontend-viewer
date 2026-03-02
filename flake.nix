{
  description = "rust flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix/monthly";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, fenix, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ fenix.overlays.default ];
        };
        deps = with pkgs; [
          webkitgtk_6_0
          gtk4
          libsoup_3
          gdk-pixbuf
          cairo
          glib
          pkg-config
        ];
      in
      {
        devShells.default =
        let
          rust-pkgs = pkgs.fenix.stable.toolchain;
        in
        pkgs.mkShell {
          buildInputs = with pkgs; [
            rust-pkgs
            rust-analyzer-nightly
          ] ++ deps;
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath deps;
        };
      }
    );
}
