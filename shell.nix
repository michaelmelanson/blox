let
  # Pinned nixpkgs, deterministic. Last updated: 2/12/21.
  # pkgs = import (fetchTarball("https://github.com/NixOS/nixpkgs/archive/a58a0b5098f0c2a389ee70eb69422a052982d990.tar.gz")) {};

  # Rolling updates, not deterministic.
  pkgs = import (fetchTarball("channel:nixpkgs-unstable")) {};

  frameworks = pkgs.darwin.apple_sdk.frameworks;
in pkgs.mkShell {
  buildInputs = [
    pkgs.bash
    pkgs.cargo
    pkgs.rustc
    pkgs.iconv
    pkgs.cargo-outdated

    frameworks.SystemConfiguration
  ];
}
