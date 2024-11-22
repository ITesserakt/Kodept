{ pkgs, lib, config, crane, fenix, inputs, ... }: let
	outputs = import ./default.nix {
		inherit pkgs crane;
		fenix = fenix.packages.x86_64-linux;
		kodept_sources = ./.;
		pegviz_sources = inputs.pegviz;
		doStaticBuild = true;
	};
in {
	packages = with pkgs; [
		xdot
		gnuplot
	];

    # Override rust src for rust_rover support and hope that nightly lib is compatible with stable one
    # This is necessary because rust rover complains about invalid lib structure for stable rust src
	env.RUST_SRC_PATH = lib.mkForce "${fenix.packages.x86_64-linux.latest.rust-src}/lib/rustlib/src/rust";

	pre-commit.hooks.clippy.enable = true;
	languages.rust = {
		enable = true;
		toolchain = outputs.toolchain;
	};

	scripts.kodept.exec = "${config.outputs.x86_64-linux.kodept}/bin/kodept $@";

	outputs = outputs.packages;
}
