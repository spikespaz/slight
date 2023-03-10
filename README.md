# Slight

Smooth Light - Backlight and LED devices control for Linux.

This is essentially a reimplimentation of `brightnessctl` and `light`. So why does it exist?

**TL;DR:** Minimize the amount of scripting you have to do when integrating with other programs or services.

The two utilities (`brightnessctl` and `light`) only perform very basic functions.
They can increment and decrement (or set) brightness levels of devices in `/sys/class/backlight` and `/sys/class/leds`.
That's great, and wonderful how simple and straightforward they are, however if you want to do anything more
complicated you'll need to write wrappers in another language (usually shell code) to take care of that.

Considering that actually using the `sysfs` ABIs for changing device brightness is so simple,
you may as well just do it with your own scripts. **Slight** exists so that you don't have to reinvent the
wheel anymore.

## Advantages

- [X] Interpolate brightness adjustments over a duration of time
- [X] Conditionally adjust brightness only if it is currently above or below the target.
- [ ] Direct integration with other programs (such as [Gammastep] or [Redshift], with hooks).
- [ ] Control brightness external monitors with DDC/CI.
- [ ] Control multiple devices at the same time, so that one command affects multiple.
- [ ] Define custom percentage curves so that brightness does not adjust linearly, but rather according to your eye's perception.

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
    # ...
  };
}
```
