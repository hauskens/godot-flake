{ ... }:
{
  projectRootFile = "flake.nix";

  programs = {
    nixfmt.enable = true;
    shfmt.enable = true;
    gdformat.enable = true;
  };
}
