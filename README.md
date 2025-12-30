# Slight

Smooth Light - Backlight and LED devices control for Linux.

This program attempts to be a reimplementation of tools such as `brightnessctl` and `light`, but with better ergonomics for scripting purposes.

## Motive

The two most common utilities (`brightnessctl` and `light`) only perform very basic functions. `clight` is in its own category with regards to feature selection.

They can all increment and decrement (or set) brightness levels of devices in `/sys/class/backlight` and `/sys/class/leds`.

For what they are, they work fine. But if you want--for example--all of your screeens to fade to black, you'll be implementing a some buggy shell code to interpolate the `brightnessctl` calls over time. Hence, `slight`, which is an excersise in foresight.

Considering that actually using the `sysfs` ABIs for changing device brightness is so simple,
you may as well just do it with your own scripts. **Slight** exists so that you don't have to reinvent the
wheel anymore.

## Features

- [X] Interpolate brightness adjustments over a duration of time
- [X] Conditionally adjust brightness only if it is currently above or below the target.
- [ ] Control brightness external monitors with DDC/CI.
- [ ] Control multiple devices at the same time,
  - [ ] with different perceived brightness levels
  - [ ] and normalized by custom curve configurations.
- [ ] Direct integration with other programs (such as [Gammastep] or [Redshift], with hooks).
- [ ] JSON with `stdio`.
- [ ] Ambient light sensor integration.

[gammastep]: https://gitlab.com/chinstrap/gammastep
[redshift]: http://jonls.dk/redshift/

## Installation

If you package this program for any distributions, please add it below!

### Generic Linux

Assuming you have Rust installed, with `$HOME/.cargo/bin` added to your environment's `PATH`:

```sh
$ cargo install slight
```

> **Note:**
>
> The binary will need to be run with `sudo` unless you install the requisite
> `udev` rules. These can be found in `backlight-90.rules` in the root of the repository.
>
> Your user must also be added to the `video` group to satisfy these rules.
>
> Copy `backlight-90.rules` to `/etc/udev/rules.d`, and add your user to the `video` group.

```sh
$ curl https://raw.githubusercontent.com/spikespaz/slight/master/90-backlight.rules -o 90-backlight.rules
$ sudo install -Dm444 90-backlight.rules -t /etc/udev/rules.d
$ sudo usermod -aG video $USER
```

### NixOS

> **Note:**
>
> Don't forget to install the `udev` rules!
>
> This can be done via the NixOS option `services.udev.extraRules` or
> `services.udev.packages`.
>
> For example, in your system configuration:
>
> ```nix
> {
>   environment.systemPackages = [pkgs.slight];
>   services.udev.packages = [pkgs.slight];
>   users.users.YOURNAME.extraGroups = ["video"];
> }
> ```

#### With Flakes

Below is an example showing how to use the overlay, so that you can use the package from `pkgs.slight` throughout your Nix configurations.

```nix
{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

    slight.url = "github:spikespaz/slight";
    slight.follows = "nixpkgs";
  };

  outputs = inputs @ {
    self,
    nixpkgs,
    ...
  }: let
    system = "x86_64-linux";

    pkgs = import nixpkgs {
      inherit system;
      overlays = [
        inputs.slight.overlays.default
      ];
    };
  in {
    packages.${system}.slight = pkgs.slight;
    # ...
  };
}
```
