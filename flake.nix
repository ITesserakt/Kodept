{
	description = "Programming language";

	inputs = {
		nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
		fenix.url = "github:nix-community/fenix";
		crane.url = "github:ipetkov/crane";
		pegviz.url = "github:fasterthanlime/pegviz";
		pegviz.flake = false;
	};

	outputs = { self, nixpkgs, fenix, crane, pegviz }: let
		system = "x86_64-linux";
		pkgs = import nixpkgs { inherit system; };
		
		outputs = import ./default.nix {
			inherit pkgs crane;
			fenix = fenix.packages.${system};
			doStaticBuild = true;
			kodept_sources = ./.;
			pegviz_sources = pegviz;
		};
	in {
		packages = outputs.packages // {
			${system}.default = outputs.packages.${system}.kodept;
		};

		devShells.${system}.default = let
            local_outputs = import ./default.nix {
                inherit pkgs crane;
                fenix = fenix.packages.${system};
                doStaticBuild = false;
                kodept_sources = ./.;
                pegviz_sources = pegviz;
                useNightly = true;
            };
		in pkgs.mkShell rec {
			packages = with pkgs; [
				xdot
				gnuplot
				pkgs.stdenv.cc
                local_outputs.toolchain
			];

			toolchain = pkgs.symlinkJoin {
				name = "kodept-toolchain";
				paths = packages;
			};
		
			shellHook = ''
				rm -f .toolchain
				ln -s ${toolchain} .toolchain
			'';
		};
	};
}
