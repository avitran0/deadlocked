{
  description = "Deadlocked development and runtime environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-26.05";
    systems.url = "github:nix-systems/default-linux";
  };

  outputs = {
    self,
    nixpkgs,
    systems,
  }: let
    eachSystem = nixpkgs.lib.genAttrs (import systems);
  in {
    devShells = eachSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};

      # Graphics and X11 libraries
      runtimeLibs = with pkgs; [
        libGL
        wayland
        libxkbcommon
        libx11
        libxcursor
        libxi
        libxrandr
        libxfixes
        libxinerama
      ];
    in {
      default = pkgs.mkShell {
        name = "deadlocked-dev-shell";

        nativeBuildInputs = with pkgs; [
          # Compiler and package manager
          pkg-config
          cargo
          rustc

          # Dev tools
          cargo-audit
          cargo-deny
          clippy
          rustfmt
          rust-analyzer
          scdoc

          # Debugging
          gdb
          strace
        ];

        # Compile-time headers and libraries
        buildInputs = runtimeLibs;

        # Runtime dynamic library resolution
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath runtimeLibs;
      };
    });

    # Code formatting via "nix fmt"
    formatter = eachSystem (system: nixpkgs.legacyPackages.${system}.alejandra);
  };
}
