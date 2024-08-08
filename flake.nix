{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    naersk,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};

      naersk' = pkgs.callPackage naersk {};
    in {
      defaultPackage = naersk'.buildPackage {
        src = ./.;
      };
      devShells = {
        treeSitterDev = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            nodejs_latest
            tree-sitter
            clang
          ];
          profile = /* bash */ ''
            export npm_config_build_from_source=true
          '';
        };
      };
    });
}
