{ pkgs, ... }:

{
  packages = with pkgs; [ texliveFull just ];
}
