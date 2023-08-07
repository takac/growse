{
  description = "git-open-rs";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    treefmt-nix.url = "github:numtide/treefmt-nix";
    flake-utils.url = "github:numtide/flake-utils";
    pre-commit-hooks.url = "github:cachix/pre-commit-hooks.nix";
  };
  outputs = { self, nixpkgs, treefmt-nix, pre-commit-hooks, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      {
        formatter = (treefmt-nix.lib.evalModule nixpkgs.legacyPackages.${system} ./treefmt.nix).config.build.wrapper;
        checks = {
          pre-commit-check = pre-commit-hooks.lib.${system}.run {
            src = ./.;
            hooks = {
              rustfmt.enable = true;
              clippy.enable = true;
              cargo-check.enable = true;
              nixpkgs-fmt.enable = true;
            };
          };
        };

        packages = {
          default = nixpkgs.legacyPackages.${system}.callPackage ./. { };
        };

        devShell = nixpkgs.legacyPackages.${system}.mkShell {
          inherit (self.checks.${system}.pre-commit-check) shellHook;
        };

        # devShell = {
        #   default = pkgsFor.${system}.mkShell {
        #     buildInputs = [ pkgsFor.${system}.git ];
        #   };
        # };
      });
}
