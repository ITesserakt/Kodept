{ pkgs ? import <nixpkgs> {} }: let
  fenix = import (fetchTarball "https://github.com/nix-community/fenix/archive/monthly.tar.gz") { };
in pkgs.mkShellNoCC rec {
	packages = with pkgs; [
		plantuml
		gnuplot
		pkgs.clangStdenv.cc
		mold
		fenix.latest.toolchain
		graphviz
		bashInteractive
		bacon
	];

	toolchain = pkgs.symlinkJoin {
		name = "kodept-toolchain";
		paths = packages;
	};

	shellHook = ''
		rm -f .toolchain
		ln -s ${toolchain} .toolchain
	'';
}
