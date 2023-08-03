{
  description = "An anyrun plugin that lets you search NixOS options.";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = inputs @ {
    nixpkgs,
    flake-parts,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      perSystem = {
        config,
        pkgs,
        ...
      }: rec {
        packages = rec {
          anyrun-nixos-options = pkgs.callPackage ./nix {};
          default = anyrun-nixos-options;
        };

        legacyPackages = packages;

        devShells.default = with pkgs;
          mkShell {
            buildInputs = [
              cargo
              clippy
              rustc
              git
              rustfmt
              rust-analyzer
            ];

            inputsFrom = [packages.default];
          };
      };
    };
}
