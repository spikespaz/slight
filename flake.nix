{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = inputs @ {
    self,
    nixpkgs,
    ...
  }: let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};
  in {
    overlays.${system} = {
      default = _: prev: {
        slight = prev.callPackage ({
          lib,
          rustPlatform,
        }:
          rustPlatform.buildRustPackage (let
            manifest = lib.importTOML ./Cargo.toml;
          in {
            pname = manifest.package.name;
            version = manifest.package.version;
            src = ./.;
            cargoHash = "sha256-frbnF/TBFau1QMOlmC1GKsM2uCBG3SdFuIscuiCqdHc=";
          })) {};
      };
    };

    packages.${system} = {
      default = self.packages.${system}.slight;
      slight = (self.overlays.${system}.default null pkgs).slight;
    };
  };
}
