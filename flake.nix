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
        # Desktop app (gpui). Untested on NixOS: needs GPU/windowing libs at runtime.
        droidkraft = pkgs.rustPlatform.buildRustPackage {
          pname = "droidkraft";
          version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).workspace.package.version;
          src = self;
          cargoLock.lockFile = ./Cargo.lock;
          cargoBuildFlags = [ "-p" "droidkraft" ];
          doCheck = false;
          nativeBuildInputs = [ pkgs.pkg-config pkgs.makeWrapper ];
          buildInputs = pkgs.lib.optionals pkgs.stdenv.isLinux (with pkgs; [ libxkbcommon wayland vulkan-loader libx11 libxcb fontconfig alsa-lib ]);
          postFixup = pkgs.lib.optionalString pkgs.stdenv.isLinux "wrapProgram $out/bin/droidkraft --prefix LD_LIBRARY_PATH : ${pkgs.lib.makeLibraryPath [ pkgs.vulkan-loader pkgs.wayland pkgs.libxkbcommon ]}";
          meta = {
            description = "Android device toolkit (desktop app)";
            homepage = "https://github.com/sorinirimies/droidkraft";
            license = pkgs.lib.licenses.mit;
            mainProgram = "droidkraft";
          };
        };
        default = droidkraft-tui;
      });
      devShells = forAll (pkgs: {
        default = pkgs.mkShell { packages = with pkgs; [ cargo rustc clippy rustfmt pkg-config nushell just ]; };
      });
    };
}
