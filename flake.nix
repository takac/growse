{
  description = "git-open-rs";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    flake-utils.url = "github:numtide/flake-utils";
    pre-commit-hooks.url = "github:cachix/pre-commit-hooks.nix";
  };
  outputs = { self, nixpkgs, treefmt-nix, systems, pre-commit-hooks, flake-utils }:
    let
      forAllSystems = nixpkgs.lib.genAttrs (import systems);
      eachSystem = f: nixpkgs.lib.genAttrs (import systems) (system: f nixpkgs.legacyPackages.${system});
      treefmtEval = forAllSystems (system: treefmt-nix.lib.evalModule nixpkgs.legacyPackages.${system} ./treefmt.nix);
    in
    {
      formatter = forAllSystems (system: treefmtEval.${system}.config.build.wrapper);
      checks = {
        pre-commit-check = forAllSystems (system: pre-commit-hooks.lib.${system}.run {
          src = ./.;
          hooks = {
            rustfmt.enable = true;
            clippy.enable = true;
            nixpkgs-fmt.enable = true;
          };
        });
      };

      packages = forAllSystems (system: {
        default = nixpkgs.legacyPackages.${system}.callPackage ./. { };
      });

      # devShell = nixpkgs.legacyPackages.${system}.mkShell {
      #    inherit (self.checks.${system}.pre-commit-check) shellHook;
      #  };
      # devShell = {
      #   default = pkgsFor.${system}.mkShell {
      #     buildInputs = [ pkgsFor.${system}.git ];
      #   };
      # };
    };
}
