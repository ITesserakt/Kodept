{
  inputs = {
    fenix.url = "github:nix-community/fenix/staging";
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    devenv.url = "github:cachix/devenv";
  };

  outputs = { self, flake-utils, naersk, nixpkgs, fenix, devenv }@inputs:
    let
      system = flake-utils.lib.system.x86_64-linux;

      pkgs = (import nixpkgs) {
        inherit system;
      };

      toolchain = fenix.packages.${system}.latest;

      toolchain_win = with fenix.packages.${system}; combine [
        minimal.rustc
        minimal.cargo
        targets.x86_64-pc-windows-gnu.latest.rust-std
      ];

      naersk' = naersk.lib.${system}.override {
        cargo = toolchain.toolchain;
        rustc = toolchain.toolchain;
      };

      naersk_win = naersk.lib.${system}.override {
        cargo = toolchain_win;
        rustc = toolchain_win;
      };
    in
    {
      packages.${system}.default = naersk'.buildPackage {
        src = self;
        nativeBuildInputs = with pkgs; [ pkgsStatic.stdenv.cc ];
        doCheck = true;
        doDoc = false;
      };

      packages.${flake-utils.lib.system.x86_64-windows}.default = naersk_win.buildPackage {
        src = self;
        strictDeps = true;
        doCheck = false;
        nativeBuildInputs = with pkgs; [ wineWowPackages.stable ];
        depsBuildBuild = with pkgs; [
          pkgsCross.mingwW64.stdenv.cc
          pkgsCross.mingwW64.windows.pthreads
        ];

        CARGO_BUILD_TARGET = "x86_64-pc-windows-gnu";
        CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUNNER = pkgs.writeScript "wine-wrapper" ''
          export WINEPREFIX = "$(mktemp -d)"
          exec wine64 $@
        '';
      };

      devShells.${system}.default = devenv.lib.mkShell {
        inherit inputs pkgs;
        modules = [
          ({ pkgs, config, ... }: {
             packages = with pkgs; [ xdot rustup graphviz ];
             pre-commit.hooks.clippy.enable = true;
             languages.rust = {
               enable = true;
               toolchain = toolchain;
             };
           })
        ];
      };
    };
}
