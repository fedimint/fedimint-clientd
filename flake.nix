{
  description = "A fedimint client daemon for server side applications to hold, use, and manage Bitcoin and ecash";

  inputs = {
    nixpkgs = {
      url = "github:nixos/nixpkgs/nixos-24.05";
    };

    flakebox = {
      url = "github:rustshop/flakebox?rev=ee39d59b2c3779e5827f8fa2d269610c556c04c8";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";

    fedimint = {
      url = "github:fedimint/fedimint?ref=v0.4.2";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flakebox,
      flake-utils,
      fedimint,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
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
          paths = [
            "Cargo.toml"
            "Cargo.lock"
            ".cargo"
            "src"
            "multimint"
            "fedimint-clientd"
            "fedimint-nwc"
            "clientd-stateless"
          ];
        };

        toolchainArgs =
          let
            llvmPackages = pkgs.llvmPackages_11;
          in
          {
            extraRustFlags = "--cfg tokio_unstable";

            components = [
              "rustc"
              "cargo"
              "clippy"
              "rust-analyzer"
              "rust-src"
            ];

            args = {
              nativeBuildInputs = [
                pkgs.wasm-bindgen-cli
                pkgs.geckodriver
                pkgs.wasm-pack
              ] ++ lib.optionals (!pkgs.stdenv.isDarwin) [ pkgs.firefox ];
            };
          };

        # all standard toolchains provided by flakebox
        toolchainsStd = flakeboxLib.mkStdFenixToolchains toolchainArgs;

        toolchainsNative = (pkgs.lib.getAttrs [ "default" ] toolchainsStd);

        toolchainNative = flakeboxLib.mkFenixMultiToolchain { toolchains = toolchainsNative; };

        commonArgs = {
          buildInputs =
            [ ]
            ++ lib.optionals pkgs.stdenv.isDarwin [ pkgs.darwin.apple_sdk.frameworks.SystemConfiguration ];
          nativeBuildInputs = [ pkgs.pkg-config ];
        };
        outputs = (flakeboxLib.craneMultiBuild { toolchains = toolchainsStd; }) (
          craneLib':
          let
            craneLib =
              (craneLib'.overrideArgs {
                pname = "flexbox-multibuild";
                src = rustSrc;
              }).overrideArgs
                commonArgs;
          in
          rec {
            workspaceDeps = craneLib.buildWorkspaceDepsOnly { };
            workspaceBuild = craneLib.buildWorkspace { cargoArtifacts = workspaceDeps; };
            fedimint-clientd = craneLib.buildPackageGroup {
              pname = "fedimint-clientd";
              packages = [ "fedimint-clientd" ];
              mainProgram = "fedimint-clientd";
            };

            fedimint-clientd-oci = pkgs.dockerTools.buildLayeredImage {
              name = "fedimint-clientd";
              contents = [ fedimint-clientd ];
              config = {
                Cmd = [ "${fedimint-clientd}/bin/fedimint-clientd" ];
              };
            };
          }
        );
      in
      {
        legacyPackages = outputs;
        packages = {
          default = outputs.fedimint-clientd;
        };
        devShells = flakeboxLib.mkShells {
          packages = with pkgs; [
            jdk21 # JDK 22 will be in $JAVA_HOME (and in javaToolchains)
            jextract # jextract (Nix package) contains a jlinked executable and bundles its own JDK 22
            (gradle.override { # Gradle 8.7 (Nix package) depends-on and directly uses JDK 21 to launch Gradle itself
              javaToolchains = [ jdk21 ];
            })
          ];
          buildInputs = commonArgs.buildInputs;
          nativeBuildInputs = [
            pkgs.mprocs
            pkgs.go
            pkgs.bun
            pkgs.bitcoind
            pkgs.clightning
            pkgs.lnd
            pkgs.esplora-electrs
            pkgs.electrs
            commonArgs.nativeBuildInputs
            fedimint.packages.${system}.devimint
            fedimint.packages.${system}.gateway-pkgs
            fedimint.packages.${system}.fedimint-pkgs
          ];
          shellHook = ''
            export JAVA_HOME="${pkgs.jdk21}"
            export RUSTFLAGS="--cfg tokio_unstable"
            export RUSTDOCFLAGS="--cfg tokio_unstable"
            export RUST_LOG="info"
          '';
        };
      }
    );
}
