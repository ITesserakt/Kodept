let
  nixpkgs = fetchTarball "https://github.com/NixOS/nixpkgs/tarball/nixos-23.11";
  pkgs = import nixpkgs { config = {}; };
in 

pkgs.mkShell {
  packages = with pkgs; [
    texliveFull
  ];
}
