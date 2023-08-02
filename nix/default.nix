{rustPlatform}:
rustPlatform.buildRustPackage {
  pname = "anyrun-nixos-options";
  version = "0.1.0";

  src = ../.;

  cargoLock = {
    lockFile = ../Cargo.lock;
    outputHashes."anyrun-interface-0.1.0" = "sha256-YrvbqM9WXr3/cgFVwc1EjIaQq4aI4DdrB1fhHfAW/d4=";
  };
}
