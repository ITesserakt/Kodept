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
			useNightly = true;
		};
	in {
		packages = outputs.packages // {
			${system}.default = outputs.packages.${system}.kodept;
		};
  };
}
