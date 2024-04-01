{ pkgs, ... }:

{
  packages = with pkgs; [ texliveFull ];

  scripts.ide.exec = ''${pkgs.jetbrains.rust-rover}/bin/rust-rover'';
}
