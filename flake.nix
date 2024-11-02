{
  inputs.nixpkgs.url = "nixpkgs/nixpkgs-unstable";

  outputs = { self, nixpkgs }: {
    devShells.x86_64-darwin.default = let
      pkgs = nixpkgs.legacyPackages.x86_64-darwin;
      frameworks = pkgs.darwin.apple_sdk.frameworks;
    in pkgs.mkShell {
      buildInputs = [
        pkgs.bash
        pkgs.cargo
        pkgs.rustc
        pkgs.iconv
        pkgs.cargo-outdated
        pkgs.tailwindcss
        frameworks.SystemConfiguration
        frameworks.CoreServices
      ];
    };

    packages.x86_64-darwin.default = let
      pkgs = nixpkgs.legacyPackages.x86_64-darwin;
    in pkgs.rustPlatform.buildRustPackage {
      pname = "blox-server";
      version = "0.1.0";
      src = ./.;
      cargoLock.lockFile = ./Cargo.lock;
      cargoBuildFlags = [ "-p" "blox-server" ];
    };
  };
}
