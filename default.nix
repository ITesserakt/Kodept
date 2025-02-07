{
	pkgs,
	fenix,
	crane,
	doStaticBuild,
	useNightly ? false,
	pegviz_sources,
	kodept_sources
}: let
    toolchainType = if useNightly then "latest" else "stable";

	toolchain = fenix.combine [
		fenix.${toolchainType}.toolchain
		(if doStaticBuild then
			fenix.targets.x86_64-unknown-linux-musl.${toolchainType}.rust-std
		 else
		 	fenix.targets.x86_64-unknown-linux-gnu.${toolchainType}.rust-std
		)
	];
	toolchain-win = fenix.combine [
	    fenix.${toolchainType}.toolchain
	    fenix.targets.x86_64-pc-windows-gnu.${toolchainType}.rust-std
	];

	craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
	craneLib-win = (crane.mkLib pkgs).overrideToolchain toolchain-win;

	commonArgs = {
		doDoc = false;

		CARGO_BUILD_TARGET = if doStaticBuild then "x86_64-unknown-linux-musl" else "x86_64-unknown-linux-gnu";
    CARGO_BUILD_RUSTFLAGS = if doStaticBuild then "-C target-feature=+crt-static" else "";
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
in rec {
	packages.x86_64-linux.pegviz = craneLib.buildPackage (commonArgs // {
		src = pegviz_sources;
	});
	packages.x86_64-linux.kodept = craneLib.buildPackage (commonArgs // {
		src = kodept_sources;
		propagatedBuildInputs = [ packages.x86_64-linux.pegviz ];
		cargoExtraArgs = "-F parallel";
	});
	packages.x86_64-windows.pegviz = craneLib-win.buildPackage (commonArgs-win // {
		src = pegviz_sources;
	});
	packages.x86_64-windows.kodept = craneLib-win.buildPackage (commonArgs-win // {
		src = kodept_sources;
		propagatedBuildInputs = [ packages.x86_64-windows.pegviz ];
		cargoExtraArgs = "-F parallel";
	});

	inherit toolchain;
}
