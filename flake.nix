{
  description = "An anyrun plugin that lets you search NixOS options.";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
      in rec {
        devShell = with pkgs;
          mkShell {
            buildInputs = [
              cargo
              clippy
              rustc
              git
              rustfmt
              rust-analyzer
            ];
          };

        packages = rec {
          anyrun-nixos-options = pkgs.callPackage ./nix {};
          default = anyrun-nixos-options;
        };

        anyrunPlugins = rec {
          anyrun-nixos-options = "${packages.default}/lib/libanyrun_nixos_options.so";
          default = anyrun-nixos-options;
        };

        legacyPackages = packages;
      }
    );
}
