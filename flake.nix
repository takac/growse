{
  description = "git-open-rs";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };
  outputs = { self, nixpkgs }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-darwin" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
      pkgsFor = nixpkgs.legacyPackages;
    in
    {
      packages = forAllSystems (system: {
        default = pkgsFor.${system}.callPackage ./. { };
      });
      # devShell = forAllSystems (system: {
      #   default = pkgsFor.${system}.mkShell {
      #     buildInputs = [ pkgsFor.${system}.git ];
      #   };
      # });
    };
}
