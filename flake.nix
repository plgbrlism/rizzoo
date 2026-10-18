{
  description = "Cross-platform Material 3 Expressive color generation";

  inputs = {
    nixpkgs.url = "https://channels.nixos.org/nixos-unstable/nixexprs.tar.zst";
  };

  outputs =
    { self, nixpkgs }:
    let
      inherit (nixpkgs.lib) genAttrs;

      cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);

      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      forEachSystem =
        perSystem:
        genAttrs systems (
          system:
          let
            pkgs = nixpkgs.legacyPackages.${system};
          in
          perSystem { inherit pkgs system; }
        );
    in
    {
      packages = forEachSystem (
        { pkgs, ... }:
        rec {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "rizzoo";
            version = cargoToml.package.version;
            src = ./.;

            cargoHash = "sha256-vtErNqgYbUjoxbAvHOZVvDZZdzIAL+RVnq2GriWd9bQ=";

            buildInputs = with pkgs; [
              pkg-config
              clang
              llvmPackages.libclang
            ];
          };
        }
      );

      devShells = forEachSystem (
        { pkgs, system }:
        {
          default = pkgs.mkShell {
            inputsFrom = [ self.packages.${system}.default ];

            packages = with pkgs; [
              rust-analyzer
              cargo-watch
            ];
          };
        }
      );

      apps = forEachSystem (
        { system, ... }:
        {
          default = {
            type = "app";
            program = "${self.packages.${system}.default}/bin/rizzoo";
          };
        }
      );
    };
}
