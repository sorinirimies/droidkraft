{
  description = "DroidKraft — Android device toolkit (TUI)";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAll = f: nixpkgs.lib.genAttrs systems (s: f nixpkgs.legacyPackages.${s});
    in {
      packages = forAll (pkgs: rec {
        droidkraft-tui = pkgs.rustPlatform.buildRustPackage {
          pname = "droidkraft-tui";
          version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).workspace.package.version;
          src = self;
          cargoLock.lockFile = ./Cargo.lock;
          cargoBuildFlags = [ "-p" "droidkraft-tui" ];
          doCheck = false;
          nativeBuildInputs = [ pkgs.pkg-config ];
          meta = {
            description = "Terminal UI for Android devices";
            homepage = "https://github.com/sorinirimies/droidkraft";
            license = pkgs.lib.licenses.mit;
            mainProgram = "droidkraft-tui";
          };
        };
        default = droidkraft-tui;
      });
      devShells = forAll (pkgs: {
        default = pkgs.mkShell { packages = with pkgs; [ cargo rustc clippy rustfmt pkg-config nushell just ]; };
      });
    };
}
