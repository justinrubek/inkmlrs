{
  inputs,
  self,
  ...
} @ part-inputs: {
  imports = [];

  perSystem = {
    pkgs,
    lib,
    system,
    inputs',
    ...
  }: let
    fenix-channel = inputs'.fenix.packages.latest;
    fenix-toolchain = fenix-channel.withComponents [
      "rustc"
      "cargo"
      "clippy"
      "rust-analysis"
      "rust-src"
      "rustfmt"
      "llvm-tools-preview"
    ];

    craneLib = inputs.crane.lib.${system}.overrideToolchain fenix-toolchain;

    common-build-args = rec {
      src = inputs.nix-filter.lib {
        root = ../.;
        include = [
          "crates"
          "Cargo.toml"
          "Cargo.lock"
        ];
      };

      pname = "inkmlrs";

      buildInputs = allBuildInputs [];
      nativeBuildInputs = allNativeBuildInputs [];
      LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
    };
    deps-only = craneLib.buildDepsOnly ({} // common-build-args);

    clippy-check = craneLib.cargoClippy ({
        cargoArtifacts = deps-only;
        cargoClippyExtraArgs = "--all-features -- --deny warnings";
      }
      // common-build-args);

    rust-fmt-check = craneLib.cargoFmt ({
        inherit (common-build-args) src;
      }
      // common-build-args);

    tests-check = craneLib.cargoNextest ({
        cargoArtifacts = deps-only;
        partitions = 1;
        partitionType = "count";
      }
      // common-build-args);

    pre-commit-hooks = inputs.pre-commit-hooks.lib.${system}.run {
      inherit (common-build-args) src;
      hooks = {
        alejandra.enable = true;
        rustfmt.enable = true;
      };
    };

    cli-package = craneLib.buildPackage ({
        pname = "inkmlrs";
        cargoArtifacts = deps-only;
        cargoExtraArgs = "--bin inkmlrs";
      }
      // common-build-args);

    devTools = with pkgs; [
      # rust tooling
      fenix-toolchain
      bacon
      rustfmt
      # version control
      cocogitto
      inputs'.bomper.packages.cli
      # misc
    ];

    extraBuildInputs = [
      pkgs.pkg-config
      pkgs.udev
      pkgs.alsaLib
      pkgs.vulkan-loader
      pkgs.xorg.libXcursor
      pkgs.xorg.libXrandr
      pkgs.xorg.libXi
      pkgs.libxkbcommon
      pkgs.wayland
      pkgs.clang
      pkgs.libGL
    ];
    extraNativeBuildInputs = [
    ];

    allBuildInputs = base: base ++ extraBuildInputs;
    allNativeBuildInputs = base: base ++ extraNativeBuildInputs;
  in rec {
    devShells.default = pkgs.mkShell rec {
      buildInputs = allBuildInputs [fenix-toolchain] ++ devTools;
      nativeBuildInputs = allNativeBuildInputs [];
      LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
      inherit (self.checks.${system}.pre-commit-hooks) shellHook;
    };

    packages = {
      default = packages.cli;
      cli = cli-package;
    };

    apps = {
      cli = {
        type = "app";
        program = "${self.packages.${system}.cli}/bin/cli";
      };
      default = apps.cli;
    };

    checks = {
      inherit pre-commit-hooks;
      clippy = clippy-check;
      tests = tests-check;
      rust-fmt = rust-fmt-check;
    };
  };
}
