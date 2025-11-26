{
  description = "A PoC for getting push notifications with Dioxus.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    devshell = {
      # NOTE: Use the latest version that provides an environment where
      # libraries are found by the linker.
      url = "github:numtide/devshell/9eee6508a16e6b63fa7b1a3ea387e2dbe321dbec";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    android-nixpkgs = {
      url = "github:tadfisher/android-nixpkgs/stable";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    git-z = {
      url = "https://flakehub.com/f/ejpcmac/git-z/*";
      inputs.nixpkgs.follows = "nixpkgs";
      # NOTE: Use the latest rust-overlay, compatible with the latest nixpkgs.
      inputs.rust-overlay.follows = "rust-overlay";
    };
  };

  outputs = { flake-parts, ... }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [ inputs.devshell.flakeModule ];
      systems = [ "x86_64-linux" "x86_64-darwin" "aarch64-darwin" ];

      perSystem = { inputs', system, ... }:
        let
          overlays = [ (import inputs.rust-overlay) ];
          pkgs = import inputs.nixpkgs { inherit system overlays; };

          rust-toolchain =
            pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

          android-sdk =
            inputs.android-nixpkgs.sdk.${system} (sdkPkgs: with sdkPkgs; [
              cmdline-tools-latest
              build-tools-34-0-0
              ndk-25-2-9519653
              platforms-android-33
            ]);

          dependencies = with pkgs; [
            at-spi2-atk
            cairo
            gdk-pixbuf
            glib
            gtk3
            harfbuzz
            libsoup_3
            openssl
            pango
            wayland
            webkitgtk_4_1
            xdotool
            zlib
          ];
        in
        {
          ######################################################################
          ##                            Devshells                             ##
          ######################################################################

          devshells =
            let
              git-z = inputs'.git-z.packages.git-z;

              language = {
                c = {
                  includes = dependencies;
                  libraries = dependencies;
                };

                rust.enableDefaultToolchain = false;
              };

              rustToolchain = version:
                if version == "stable" then
                  rust-toolchain
                else if version == "nightly" then
                  (pkgs.rust-bin.nightly."2025-11-01".minimal.override {
                    extensions = [ "llvm-tools" ];
                  })
                else throw "the Rust version must be `stable` or `nightly`";

              buildToolchain = version: with pkgs; [
                dioxus-cli
                nodejs
                (rustToolchain version)
              ] ++ lib.optionals (!stdenv.isDarwin) [
                clang
              ];

              checkToolchain = with pkgs; [
                cargo-hack
                cargo-nextest
                committed
                eclint
                nixpkgs-fmt
                nodePackages.prettier
                taplo
                typos
              ];

              nightlyCheckToolchain = with pkgs; [
                cargo-udeps
              ] ++ lib.optionals (!stdenv.isDarwin) [
                cargo-llvm-cov
              ];

              ideToolchain = with pkgs; [
                nixd
                rust-analyzer
              ];

              devTools = with pkgs; [
                bacon
                cargo-bloat
                cargo-outdated
                git
                git-z
                gitflow
              ];

              buildEnv = [
                {
                  name = "ANDROID_HOME";
                  value = "${android-sdk}/share/android-sdk";
                }
                {
                  name = "GRADLE_OPTS";
                  value =
                    "-Dorg.gradle.project.android.aapt2FromMavenOverride=" +
                    "${android-sdk}/share/android-sdk/build-tools/34.0.0/aapt2";
                }
                {
                  name = "JAVA_HOME";
                  value = pkgs.jdk17.home;
                }
              ];

              devEnv = [
                {
                  name = "RUSTFLAGS";
                  value = "-Clink-arg=-fuse-ld=${pkgs.mold}/bin/mold";
                }
              ];

              ideEnv = [
                {
                  name = "NIX_PATH";
                  value = "nixpkgs=${inputs.nixpkgs}";
                }
                {
                  name = "TYPOS_LSP_PATH";
                  value = "${pkgs.typos-lsp}/bin/typos-lsp";
                }
              ];

              nightlyEnv = [
                {
                  name = "HAS_RUST_NIGHTLY";
                  value = "true";
                }
              ];
            in
            {
              default = { extraModulesPath, ... }: {
                imports = [
                  "${extraModulesPath}/language/c.nix"
                  "${extraModulesPath}/language/rust.nix"
                ];

                name = "dioxus-ntf-poc";

                motd = ''

                  {202}🔨 Welcome to the dioxus-ntf-poc devshell!{reset}
                '';

                inherit language;

                packages =
                  buildToolchain "stable"
                  ++ checkToolchain
                  ++ ideToolchain
                  ++ devTools;

                env =
                  buildEnv
                  ++ devEnv
                  ++ ideEnv;

                commands = [
                  # Pass-through commands to make some cargo extensions run with
                  # a different toolchain.
                  {
                    name = "cargo-llvm-cov";
                    command = "nix develop -L .#rust-nightly -c cargo $@";
                  }
                  {
                    name = "cargo-udeps";
                    command = "nix develop -L .#rust-nightly -c cargo $@";
                  }
                  {
                    name = "coverage-report";
                    command = ''
                      nix develop -L .#rust-nightly -c \
                        cargo llvm-cov nextest --branch --open
                    '';
                  }
                  {
                    name = "live-coverage";
                    command = ''
                      nix develop -L .#rust-nightly -c bacon coverage
                    '';
                  }
                ];
              };

              # Devshell to run tools with a nightly toolchain.
              rust-nightly = {
                name = "Rust Nightly";

                packages =
                  buildToolchain "nightly"
                  ++ nightlyCheckToolchain;

                env =
                  nightlyEnv;
              };

              ci = { extraModulesPath, ... }: {
                imports = [
                  "${extraModulesPath}/language/c.nix"
                  "${extraModulesPath}/language/rust.nix"
                ];

                name = "dioxus-ntf-poc CI";

                inherit language;

                packages =
                  buildToolchain "stable"
                  ++ checkToolchain;

                env =
                  buildEnv;
              };

              ci-nightly = { extraModulesPath, ... }: {
                imports = [
                  "${extraModulesPath}/language/c.nix"
                  "${extraModulesPath}/language/rust.nix"
                ];

                name = "dioxus-ntf-poc CI (Rust Nightly)";

                inherit language;

                packages =
                  buildToolchain "nightly"
                  ++ checkToolchain
                  ++ nightlyCheckToolchain;

                env =
                  buildEnv
                  ++ nightlyEnv;
              };
            };
        };
    };
}
