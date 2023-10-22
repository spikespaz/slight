{
# Must be provided in `callPackage` for accuracy.
sourceRoot ? ./.., platforms ? [ "x86_64-linux" ],
#
lib, rustPlatform, coreutils
#
}:
let manifest = lib.importTOML "${sourceRoot}/Cargo.toml";
in rustPlatform.buildRustPackage {
  pname = manifest.package.name;
  version = manifest.package.version;
  src = lib.cleanSourceWith {
    src = sourceRoot;
    filter = lib.mkSourceFilter sourceRoot [
      lib.defaultSourceFilter
      lib.rustSourceFilter
    ];
  };
  cargoLock.lockFile = "${sourceRoot}/Cargo.lock";

  postPatch = ''
    substituteInPlace 90-backlight.rules \
      --replace '/bin/chgrp' '${coreutils}/bin/chgrp' \
      --replace '/bin/chmod' '${coreutils}/bin/chmod'
  '';

  postInstall = ''
    install -Dm444 90-backlight.rules -t $out/etc/udev/rules.d
  '';

  meta = {
    inherit (manifest.package) description homepage;
    license = lib.licenses.mit;
    maintainers = [ lib.maintainers.spikespaz ];
    inherit platforms;
    mainProgram = manifest.package.name;
  };
}
