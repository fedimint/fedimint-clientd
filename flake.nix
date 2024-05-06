{
  description =
    "A fedimint client daemon for server side applications to hold, use, and manage Bitcoin and ecash";

  inputs = {
    nixpkgs = { url = "github:nixos/nixpkgs/nixos-23.11"; };

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flakebox = {
      url = "github:flakebox/flakebox?rev=226d584e9a288b9a0471af08c5712e7fac6f87dc";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.fenix.follows = "fenix";
    };

    flake-utils = { url = "github:numtide/flake-utils"; };

    fedimint = {
      url =
        "github:fedimint/fedimint?rev=a41e3a7e31ce0f26058206a04f1cd49ef2b12fe3";
    };
  };

  outputs = { self, nixpkgs, flakebox, fenix, flake-utils, fedimint }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = fedimint.overlays.fedimint;
        };
        lib = pkgs.lib;
        flakeboxLib = flakebox.lib.${system} { };

        toolchainArgs =
          let llvmPackages = pkgs.llvmPackages_11;
          in {
            extraRustFlags = "--cfg tokio_unstable";

            components = [ "rustc" "cargo" "clippy" "rust-analyzer" "rust-src" ];

            args = {
              nativeBuildInputs =
                [ pkgs.wasm-bindgen-cli pkgs.geckodriver pkgs.wasm-pack ]
                  ++ lib.optionals (!pkgs.stdenv.isDarwin) [ pkgs.firefox ];
            };
          } // lib.optionalAttrs pkgs.stdenv.isDarwin {
            # on Darwin newest stdenv doesn't seem to work
            # linking rocksdb
            stdenv = pkgs.clang11Stdenv;
            clang = llvmPackages.clang;
            libclang = llvmPackages.libclang.lib;
            clang-unwrapped = llvmPackages.clang-unwrapped;
          };

        # all standard toolchains provided by flakebox
        toolchainsStd = flakeboxLib.mkStdFenixToolchains toolchainArgs;

        toolchainsNative = (pkgs.lib.getAttrs [ "default" ] toolchainsStd);

        toolchainNative =
          flakeboxLib.mkFenixMultiToolchain { toolchains = toolchainsNative; };

        commonArgs = {
          buildInputs = [ ] ++ lib.optionals pkgs.stdenv.isDarwin
            [ pkgs.darwin.apple_sdk.frameworks.SystemConfiguration ];
          nativeBuildInputs = [ pkgs.pkg-config ];
        };

        commonSrc = builtins.path { path = ./.; name = "fedimint-clientd"; };

        filterWorkspaceDepsBuildFilesRegex = [ "Cargo.lock" "Cargo.toml" ".cargo" ".cargo/.*" ".config" ".config/.*" ".*/Cargo.toml" ];

        filterSrcWithRegexes = regexes: src:
          let
            basePath = toString src + "/";
          in
          lib.cleanSourceWith {
            filter = (path: type:
              let
                relPath = lib.removePrefix basePath (toString path);
                includePath =
                  (type == "directory") ||
                  lib.any
                    (re: builtins.match re relPath != null)
                    regexes;
              in
              # uncomment to debug:
                # builtins.trace "${relPath}: ${lib.boolToString includePath}"
              includePath
            );
            inherit src;
          };

        # Filter only files needed to build project dependencies
        #
        # To get good build times it's vitally important to not have to
        # rebuild derivation needlessly. The way Nix caches things
        # is very simple: if any input file changed, derivation needs to
        # be rebuild.
        #
        # For this reason this filter function strips the `src` from
        # any files that are not relevant to the build.
        #
        # Like `filterWorkspaceFiles` but doesn't even need *.rs files
        # (because they are not used for building dependencies)
        filterWorkspaceDepsBuildFiles = src: filterSrcWithRegexes filterWorkspaceDepsBuildFilesRegex src;


        # Filter only files relevant to building the workspace
        filterWorkspaceBuildFiles = src: filterSrcWithRegexes (filterWorkspaceDepsBuildFilesRegex ++ [ ".*\.rs" ]) src;

        outputs = (flakeboxLib.craneMultiBuild { toolchains = toolchainsStd; })
          (craneLib':
            let
              craneLib = (craneLib'.overrideArgs {
                pname = "flexbox-multibuild";
                src = filterWorkspaceBuildFiles commonSrc;
              }).overrideArgs commonArgs;
            in
            rec {
              workspaceDeps = craneLib.buildWorkspaceDepsOnly { };

              workspaceBuild =
                craneLib.buildWorkspace { cargoArtifacts = workspaceDeps; };

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
            });
      in
      {
        legacyPackages = outputs;
        packages = { default = outputs.fedimint-clientd; };
        devShells = flakeboxLib.mkShells {
          packages = [ ];
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
            export RUSTFLAGS="--cfg tokio_unstable"
            export RUSTDOCFLAGS="--cfg tokio_unstable"
            export RUST_LOG="info"
          '';
        };
      });
}
