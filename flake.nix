{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default-linux";
    nixfmt.url = "github:serokell/nixfmt";
  };

  outputs = inputs@{ self, nixpkgs, systems, nixfmt, ... }:
    let
      inherit (nixpkgs) lib;
      eachSystem = lib.genAttrs (import systems);
      pkgsFor = eachSystem (system: import nixpkgs { localSystem = system; });
    in {
      overlays = {
        default = final: _: {
          slight = final.callPackage ({ lib, rustPlatform, coreutils, }:
            rustPlatform.buildRustPackage
            (let manifest = lib.importTOML ./Cargo.toml;
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
            })) { };
        };
      };

      packages = eachSystem (system: {
        default = self.packages.${system}.slight;
        slight = (self.overlays.default pkgsFor.${system} null).slight;
      });

      homeManagerModules = {
        gammastep-hook = import ./nix/hm-modules/gammastep-hook.nix {
          inherit self;
          programName = "gammastep";
        };
        redshift-hook = import ./nix/hm-modules/gammastep-hook.nix {
          inherit self;
          programName = "redshift";
        };
      };

      formatter = eachSystem (system: nixfmt.packages.${system}.default);
    };
}
