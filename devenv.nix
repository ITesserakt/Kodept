{ pkgs, lib, config, crane, fenix, inputs, ... }: let
	toolchain = with fenix.packages.x86_64-linux; combine [
		stable.toolchain
		targets.x86_64-unknown-linux-musl.stable.rust-std
	];
	toolchain-win = with fenix.packages.x86_64-linux; combine [
	    stable.toolchain
	    targets.x86_64-pc-windows-gnu.stable.rust-std
	];
	
	craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
	craneLib-win = (crane.mkLib pkgs).overrideToolchain toolchain-win;

	commonArgs = {
		doDoc = false;
		
		CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
        CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
	};
	commonArgs-win = {
		strictDeps = true;
		doCheck = false;
		doDoc = false;

		CARGO_BUILD_TARGET = "x86_64-pc-windows-gnu";

		depsBuildBuild = with pkgs.pkgsCross.mingwW64; [
			stdenv.cc
			windows.pthreads
		];
	};

	pegviz_sources = lib.cleanSourceWith {
		src = inputs.pegviz;
		filter = path: type:
			(lib.hasSuffix "\.js" path)  		  ||
			(lib.hasSuffix "\.css" path) 		  ||
			(craneLib.filterCargoSources path type);
	};
	kodept_sources = lib.cleanSourceWith {
		src = ./.;
		filter = path: type:
			(lib.hasSuffix "\.pest" path )         ||
			(craneLib.filterCargoSources path type);
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
		toolchain = toolchain;
	};

	scripts.kodept.exec = "${config.outputs.x86_64-linux.kodept}/bin/kodept $@";

	outputs = rec {
		x86_64-linux.pegviz = craneLib.buildPackage (commonArgs // {
			src = pegviz_sources;
		});
		x86_64-linux.kodept = craneLib.buildPackage (commonArgs // {
			src = kodept_sources;
			buildInputs = [ x86_64-linux.pegviz ];
            cargoExtraArgs = "-F parallel";
		});
		x86_64-windows.pegviz = craneLib-win.buildPackage (commonArgs-win // {
			src = pegviz_sources;
		});
		x86_64-windows.kodept = craneLib-win.buildPackage (commonArgs-win // {
			src = kodept_sources;
			buildInputs = [ x86_64-windows.pegviz ];
			cargoExtraArgs = "-F parallel";
		});
	};
}
