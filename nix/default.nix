sourceRoot:
{ lib, rustPlatform, coreutils, }:
let manifest = lib.importTOML "${sourceRoot}/Cargo.toml";
in rustPlatform.buildRustPackage {
  pname = manifest.package.name;
  version = manifest.package.version;
  src = sourceRoot;
  cargoLock.lockFile = "${sourceRoot}/Cargo.lock";

  postPatch = ''
    substituteInPlace 90-backlight.rules \
      --replace '/bin/chgrp' '${coreutils}/bin/chgrp' \
      --replace '/bin/chmod' '${coreutils}/bin/chmod'
  '';

  postInstall = ''
    install -Dm444 90-backlight.rules -t $out/etc/udev/rules.d
  '';
}
