{
  description = "deadlocked dev shell for Nix users";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    systems.url = "github:nix-systems/default-linux";
  };

  outputs = inputs @ {
    self,
    nixpkgs,
    systems,
    ...
  }: let
    # Build outputs for each system
    eachSystem = nixpkgs.lib.genAttrs (import systems);
    pkgsFor = system:
      import nixpkgs {
        inherit system;
        overlays = [];
      };
  in {
    # Development shell
    devShells = eachSystem (system: {
      default = (pkgsFor system).callPackage ./nix/shell.nix {};
    });

    # Code formatter
    formatter.x86_64-linux = inputs.nixpkgs.legacyPackages.x86_64-linux.alejandra;
  };
}
