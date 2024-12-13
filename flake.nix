{
  inputs = {
    nixpkgs.url = "nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem(system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in {
        devShells.default = let
          frameworks = pkgs.darwin.apple_sdk.frameworks;
        in pkgs.mkShell {
          buildInputs = [
            pkgs.asciidoctor
            pkgs.bash
            pkgs.cargo
            pkgs.cargo-outdated
            pkgs.git
            pkgs.iconv
            pkgs.rustc
            pkgs.tailwindcss
            pkgs.tree-sitter
            pkgs.lldb
            frameworks.SystemConfiguration
            frameworks.CoreServices
          ];
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "blox-server";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          cargoBuildFlags = [ "-p" "blox-server" ];
        };

        packages.docs = pkgs.runCommand "docs" {
            buildInputs = [ pkgs.asciidoctor ];
        } ''
            asciidoctor ${./.}/docs/index.adoc -D $out
        '';
      }
    );
}
