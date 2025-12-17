{ pkgs ? import <nixpkgs> {} }: let
	toolchain = pkgs.rust.packages.stable;
in pkgs.mkShellNoCC rec {
	packages = with pkgs; [
		toolchain.cargo
		toolchain.rustc
		toolchain.rustfmt
		toolchain.clippy
		
		gnuplot
		pkgs.clangStdenv.cc
		graphviz
	];

	RUST_SRC_PATH = toolchain.rustPlatform.rustLibSrc;
	RUST_TOOLCHAIN_PATH = pkgs.symlinkJoin {
    name = "kodept-toolchain";
    paths = packages;
	};

	# shellHook = use-mold.moldHook pkgs.mold;
}
