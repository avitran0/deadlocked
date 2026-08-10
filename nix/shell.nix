{pkgs}: let
  # Graphics and X11 libraries
  baseLibs = with pkgs; [
    libx11
    libxcursor
    libxkbcommon
    libGL
    libxi
  ];
in
  pkgs.mkShell {
    name = "deadlocked-dev-shell";

    nativeBuildInputs = with pkgs;
      [
        # Rust toolchain
        cargo
        rustc
        scdoc
        # Runtime dependencies
        wayland
        # Development tools
        cargo-audit
        cargo-deny
        pkg-config
        clippy
        rust-analyzer
        rustfmt
        strace
        gdb
      ]
      ++ baseLibs;

    LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath baseLibs;
  }
