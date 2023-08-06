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
      eachSystem = f: nixpkgs.lib.genAttrs (import systems) (system: f nixpkgs.legacyPackages.${system});
      treefmtEval = eachSystem (pkgs: treefmt-nix.lib.evalModule pkgs ./treefmt.nix);
    in
    {
      formatter = eachSystem (pkgs: treefmtEval.${pkgs.system}.config.build.wrapper);
      checks = {
        pre-commit-check = eachSystem (pkgs: pre-commit-hooks.lib.${pkgs.system}.run {
          src = ./.;
          hooks = {
            rustfmt.enable = true;
            clippy.enable = true;
            nixpkgs-fmt.enable = true;
          };
        });
      };

      defaultPackage = eachSystem (pkgs: nixpkgs.legacyPackages.${pkgs.system}.callPackage ./. { });

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
