{
  description = "growse";
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
              # FIXME not working
              cargo-check.enable = false;
              nixpkgs-fmt.enable = true;
              # FIXME not working
              clippy = {
                enable = false;
                # args = "--all-targets --all-features -- -D warnings";
                settings = {
                  #
                  offline = true;
                  # allFeatures = true;
                  # denyWarnings = true;
                };
              };
            };
          };
        };

        packages = {
          default = nixpkgs.legacyPackages.${system}.callPackage ./. { };
        };

        devShells = {
          defualt = nixpkgs.legacyPackages.${system}.mkShell {
            inherit (self.checks.${system}.pre-commit-check) shellHook;
            # buildInputs = [ nixpkgs.legacyPackages.${system}.git ];
          };
        };

      });
}
