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
    overlays = {
      default = _: prev: {
        slight = prev.callPackage ({
          lib,
          rustPlatform,
          coreutils,
        }:
          rustPlatform.buildRustPackage (let
            manifest = lib.importTOML ./Cargo.toml;
          in {
            pname = manifest.package.name;
            version = manifest.package.version;
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;

            postPatch = ''
              substituteInPlace 90-backlight.rules \
                --replace '/bin/chgrp' '${coreutils}/bin/chgrp' \
                --replace '/bin/chmod' '${coreutils}/bin/chmod'
            '';

            postInstall = ''
              install -Dm444 90-backlight.rules -t $out/etc/udev/rules.d
            '';
          })) {};
      };
    };

    packages.${system} = {
      default = self.packages.${system}.slight;
      slight = (self.overlays.default null pkgs).slight;
    };
  };
}
