{ sourceRoot ? ../., lib, rustPlatform, coreutils }:
let manifest = lib.importTOML "${sourceRoot}/Cargo.toml";
in rustPlatform.buildRustPackage {
  pname = manifest.package.name;
  version = manifest.package.version;

  src = lib.cleanSource sourceRoot;
  cargoLock.lockFile = "${sourceRoot}/Cargo.lock";
  strictDeps = true;

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
    license = with lib.licenses; [ mit asl20 ];
    maintainers = [ lib.maintainers.spikespaz ];
    platforms = lib.platforms.linux;
    mainProgram = manifest.package.name;
  };
}
