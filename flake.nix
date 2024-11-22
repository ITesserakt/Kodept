{
	inputs.nixpkgs.url = "github:NixOS/nixpks/nixpkgs-unstable";
	inputs.fenix.url = "github:nix-community/fenix";
	inputs.pegviz.url = "github:fasterthanlime/pegviz";
	inputs.pegviz.flake = false;

	outputs = { nixpkgs, inputs, ... }: let
		system = "x86_64-linux";
		pkgs = import nixpkgs { inherit system; };
		
		packages = import ./default.nix {
			inherit pkgs;
			fenix = inputs.fenix.packages.${system};
			crane = inputs.crane;
			doStaticBuild = false;
			kodept_sources = ./.;
			pegviz_sources = inputs.pegviz;
		};
	in {
		inherit packages;
	};
}
