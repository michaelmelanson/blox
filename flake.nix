{
  inputs.nixpkgs.url = "nixpkgs/nixpkgs-unstable";

  outputs = { self, nixpkgs }: let
    pkgs = nixpkgs.legacyPackages.x86_64-darwin;
  in {
    devShells.x86_64-darwin.default = let
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

    packages.x86_64-darwin.default = pkgs.rustPlatform.buildRustPackage {
      pname = "blox-server";
      version = "0.1.0";
      src = ./.;
      cargoLock.lockFile = ./Cargo.lock;
      cargoBuildFlags = [ "-p" "blox-server" ];
    };

    packages.x86_64-darwin.docs = pkgs.runCommand "docs" {
        buildInputs = [ pkgs.asciidoctor ];
    } ''
        asciidoctor ${./.}/docs/index.adoc -D $out
    '';
  };
}
