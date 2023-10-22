{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default-linux";
    bird-nix-lib.url = "github:spikespaz/bird-nix-lib";
    nixfmt.url = "github:serokell/nixfmt";
  };

  outputs = { self, nixpkgs, bird-nix-lib, systems, nixfmt }:
    let
      lib = nixpkgs.lib.extend (bird-nix-lib.lib.overlay);
      eachSystem = lib.genAttrs (import systems);
      pkgsFor = eachSystem (system: import nixpkgs { localSystem = system; });
    in {
      overlays = {
        default = final: _: {
          slight = final.callPackage (import ./nix/default.nix) {
            sourceRoot = self.outPath;
            platforms = import systems;
            inherit lib;
          };
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
