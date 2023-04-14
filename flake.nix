{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = inputs @ {
    self,
    nixpkgs,
    ...
  }: let
    inherit (nixpkgs) lib;
    systems = [
      # Tier 1
      "x86_64-linux"
      # Tier 2
      "aarch64-linux"
      # "x86_64-darwin"
      # Tier 3
      "armv6l-linux"
      "armv7l-linux"
      "i686-linux"
      "mipsel-linux"

      # Other platforms with sufficient support in stdenv which is not formally
      # mandated by their platform tier.
      # "aarch64-darwin"
      "armv5tel-linux"
      "powerpc64le-linux"
      "riscv64-linux"

      # "x86_64-freebsd" is excluded because it is mostly broken
    ];
    pkgsFor =
      builtins.listToAttrs
      (map (system: {
          name = system;
          value = nixpkgs.legacyPackages.${system};
        })
        systems);
    mapSystems = fn: builtins.mapAttrs fn pkgsFor;
  in {
    overlays = {
      default = final: _: {
        slight = final.callPackage ({
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

    packages = mapSystems (pkgs: system: {
      default = self.packages.${system}.slight;
      slight = (self.overlays.default null pkgs).slight;
    });
  };
}
