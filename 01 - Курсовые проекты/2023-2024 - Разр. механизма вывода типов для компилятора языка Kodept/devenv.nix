{ pkgs, lib, config, inputs, ... }:

{
    packages = with pkgs; [ texliveFull just daemon ];

    scripts.ide.exec = ''${pkgs.daemon}/bin/daemon ${pkgs.gnome-latex}/bin/gnome-latex'';
}
