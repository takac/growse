{ pkgs ? import <nixpkgs> { } }:
pkgs.rustPlatform.buildRustPackage {
  pname = "growse";
  version = "0.1";
  cargoLock.lockFile = ./Cargo.lock;
  src = pkgs.lib.cleanSource ./.;
  nativeBuildInputs = [ pkgs.pkgconfig ];
  buildInputs = (
    if (pkgs.stdenv.isDarwin) then
      [
        pkgs.openssl
        pkgs.darwin.apple_sdk.frameworks.Security
      ]
    else
      [
        pkgs.openssl
      ]
  );
}

