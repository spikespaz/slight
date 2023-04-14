{
  self,
  # Will be used in the path of the script in `~/.config`,
  # as the name of the script in `/nix/store`, and as part of the path
  # to the options. See `cfg` below.
  programName,
}: {
  config,
  pkgs,
  lib,
  ...
}: let
  inherit (lib) types;
  cfg = config.services.${programName}.slight.brightnessHook;
  flakePackages = self.packages.${pkgs.stdenv.hostPlatform.system};
in {
  options = let
    mkBrightnessOption = default: period:
      lib.mkOption {
        type = types.ints.unsigned;
        inherit default;
        description = lib.mdDoc ''
          Brightness of the display during the ${period}, as a percent.
        '';
      };
    mkDurationOption = default: fromPeriod: toPeriod:
      lib.mkOption {
        type = types.strMatching ''[0-9]+(ms|ds|s|m)'';
        inherit default;
        description = lib.mdDoc ''
          The duration of time over which to inperpolate a change in brightness
          when changing from ${fromPeriod} to ${toPeriod}.
        '';
      };
  in {
    services.${programName}.slight.brightnessHook = {
      enable = lib.mkEnableOption "${programName} brightness hook for slight";
      slightPackage = lib.mkPackageOption flakePackages "slight" {};
      brightness.day = mkBrightnessOption 85 "day";
      brightness.transition = mkBrightnessOption 55 "transition period";
      brightness.night = mkBrightnessOption 25 "night";
      interpDur.dayFromTransition = mkDurationOption "5s" "transition period" "day";
      interpDur.transitionFromDay = mkDurationOption "5s" "day" "transition period";
      interpDur.nightFromTransition = mkDurationOption "10s" "transition period" "night";
      interpDur.transitionFromNight = mkDurationOption "20s" "night" "transition period";
    };
  };
  config = let
    slightExe = lib.getExe cfg.slightPackage;
    hookScript = pkgs.writeShellScript "${programName}-brightness" (with cfg; ''
      set -eu

      exec >> /tmp/redshift-hooks.log 2>&1

      if [ "$1" = 'period-changed' ]; then
        case "$3" in
          daytime)
            target="${toString brightness.day}%"
            case "$2" in
                transition)
                  ${slightExe} set -I "$target" -t ${interpDur.dayFromTransition}
                  ;;
                night|none)
                  ${slightExe} set "$target"
                  ;;
                *)
                  echo "unrecognized: $2"
                  ;;
            esac
            ;;
          transition)
            target="${toString brightness.transition}%"
            case "$2" in
                daytime)
                  ${slightExe} set -D "$target" -t ${interpDur.transitionFromDay}
                  ;;
                night)
                  ${slightExe} set -I "$target" -t ${interpDur.transitionFromNight}
                  ;;
                none)
                  ${slightExe} set "$target"
                  ;;
                *)
                  echo "unrecognized: $2"
                  ;;
            esac
            ;;
          night)
            target="${toString brightness.night}%"
            case "$2" in
              transition)
                ${slightExe} set -D "$target" -t ${interpDur.nightFromTransition}
                ;;
              daytime|none)
                ${slightExe} set "$target"
                ;;
              *)
                echo "unrecognized: $2"
                ;;
            esac
            ;;
        esac
      fi
    '');
  in
    lib.mkIf cfg.enable {
      # Gammastep and redshift use the same path, and have the same hook API.
      xdg.configFile."${programName}/hooks/brightness.sh" = {
        executable = true;
        # <https://wiki.archlinux.org/title/redshift#Use_real_screen_brightness>
        source = hookScript.outPath;
      };
    };
}
