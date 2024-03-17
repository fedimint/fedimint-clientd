{
  description = "description = A fedimint client demon for server side applications to hold, use, and manage Bitcoin";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.11";

    flakebox = {
      url = "github:rustshop/flakebox";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    fedimint = {
      url =
        "github:fedimint/fedimint?rev=9d552fdf82f4af429165a1fd409615809ada4058";
    };

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flakebox, flake-utils, fedimint }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { 
          inherit system; 
          overlays = fedimint.overlays.fedimint;
        };
        lib = pkgs.lib;
        flakeboxLib = flakebox.lib.${system} { };
        rustSrc = flakeboxLib.filterSubPaths {
          root = builtins.path {
            name = "fedimint-clientd";
            path = ./.;
          };
          paths = [ "Cargo.toml" "Cargo.lock" ".cargo" "src" ];
        };

        toolchainArgs = {
          extraRustFlags = "--cfg tokio_unstable";
        } // lib.optionalAttrs pkgs.stdenv.isDarwin {
          # on Darwin newest stdenv doesn't seem to work
          # linking rocksdb
          stdenv = pkgs.clang11Stdenv;
        };

        # all standard toolchains provided by flakebox
        toolchainsStd =
          flakeboxLib.mkStdFenixToolchains toolchainArgs;

        toolchainsNative = (pkgs.lib.getAttrs
          [
            "default"
          ]
          toolchainsStd
        );

        toolchainNative = flakeboxLib.mkFenixMultiToolchain {
          toolchains = toolchainsNative;
        };

        commonArgs = {
          buildInputs = [
            pkgs.openssl
          ] ++ lib.optionals pkgs.stdenv.isDarwin [
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
          ];
          nativeBuildInputs = [
            pkgs.pkg-config
          ];
        };
        outputs = (flakeboxLib.craneMultiBuild { toolchains = toolchainsStd; }) (craneLib':
          let
            craneLib = (craneLib'.overrideArgs {
              pname = "flexbox-multibuild";
              src = rustSrc;
            }).overrideArgs commonArgs;
          in
          rec {
            workspaceDeps = craneLib.buildWorkspaceDepsOnly { };
            workspaceBuild =
              craneLib.buildWorkspace { cargoArtifacts = workspaceDeps; };
            fedimint-clientd = craneLib.buildPackageGroup
              { pname = "fedimint-clientd"; packages = [ "fedimint-clientd" ]; mainProgram = "fedimint-clientd"; };
          });
      in
      {
        legacyPackages = outputs;
        packages = {
          inherit (flakeboxLib.craneMultiBuild) gatewayd fedimint-dbtool gateway-cli fedimint-cli fedimintd fedimint-load-test-tool;
          inherit (flakeboxLib.craneMultiBuild) client-pkgs gateway-pkgs fedimint-pkgs devimint;
        };

        devShells = flakeboxLib.mkShells (commonArgs // {
          toolchain = toolchainNative;
          nativeBuildInputs = commonArgs.nativeBuildInputs ++ [
            pkgs.mprocs
            fedimint.packages.${system}.devimint
            fedimint.packages.${system}.gateway-pkgs
            fedimint.packages.${system}.fedimint-pkgs
          ];
        });
      });
}
