{ pkgs, ... }:
{
  # Used to find the project root
  projectRootFile = "flake.nix";
  programs.rustfmt.enable = true;
  programs.nixpkgs-fmt.enable = true;
}

