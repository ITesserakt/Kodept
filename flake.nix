{
  inputs = {
    fenix.url = "github:nix-community/fenix";
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, flake-utils, naersk, nixpkgs, fenix }:
    let
      system = flake-utils.lib.system.x86_64-linux;
    
      pkgs = (import nixpkgs) {
        inherit system;
      };

      toolchain = with fenix.packages.${system}; combine [
        minimal.rustc
        minimal.cargo
        targets.x86_64-unknown-linux-musl.latest.rust-std
      ];

      toolchain_win = with fenix.packages.${system}; combine [
        minimal.rustc
        minimal.cargo
        targets.x86_64-pc-windows-gnu.latest.rust-std
      ];

      naersk' = naersk.lib.${system}.override {
        cargo = toolchain;
        rustc = toolchain;
      };

      naersk_win = naersk.lib.${system}.override {
        cargo = toolchain_win;
        rustc = toolchain_win;
      };
    in rec {
      packages.${flake-utils.lib.system.x86_64-linux}.default = naersk'.buildPackage {
        src = ./.;
        nativeBuildInputs = with pkgs; [ pkgsStatic.stdenv.cc ];
        doCheck = true;
        
        CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
        CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
      };

      packages.${flake-utils.lib.system.x86_64-windows}.default = naersk_win.buildPackage {
        src = ./.;
        strictDeps = true;
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

      # For `nix develop`:
      devShells.${system}.default = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [ rustc cargo ];
      };
    };
}
