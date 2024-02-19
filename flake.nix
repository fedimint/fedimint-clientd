{
  description = "A fedimint http client";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.11";

    flakebox = {
      url = "github:rustshop/flakebox";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flakebox, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        flakeboxLib = flakebox.lib.${system} { };
        toolchainsStd = flakeboxLib.mkStdFenixToolchains;
        rustSrc = flakeboxLib.filterSubPaths {
          root = builtins.path {
            name = "fedimint-http";
            path = ./.;
          };
          paths = [ "Cargo.toml" "Cargo.lock" ".cargo" "src" ];
        };

        outputs = (flakeboxLib.craneMultiBuild { }) (craneLib':
          let
            craneLib = (craneLib'.overrideArgs {
              pname = "flexbox-multibuild";
              src = rustSrc;
            });
          in rec {
            workspaceDeps = craneLib.buildWorkspaceDepsOnly { };
            workspaceBuild =
              craneLib.buildWorkspace { cargoArtifacts = workspaceDeps; };
          });
      in {
        legacyPackages = outputs;
        devShells = flakeboxLib.mkShells {
          packages = [ ];
          buildInputs = [
            pkgs.just
            pkgs.starship
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
            pkgs.pkg-config
          ];
          shellHook = ''
            eval "$(starship init bash)"
            export RUSTFLAGS="--cfg tokio_unstable"
          '';
        };
      });
}
