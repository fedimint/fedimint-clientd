# save this as shell.nix
{
  pkgs ? import <nixpkgs> { },
}:

pkgs.mkShell { packages = [ pkgs.hello ]; }
