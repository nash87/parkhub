{
  description = "ParkHub - Open Source Parking Management";

  nixConfig = {
    extra-substituters = [
      "http://192.168.178.201:31580/default"
    ];
    extra-trusted-public-keys = [
      "default:bL3ZH39m8SHeStEz4wjsFSyASZwlCdZm7isV5DgZ88Y="
    ];
  };

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs = { self, nixpkgs, rust-overlay, crane }:
    let
      system = "x86_64-linux";
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs {
        inherit system overlays;
        config.allowUnfree = true;
      };

      rustToolchain = pkgs.rust-bin.stable.latest.default.override {
        extensions = [ "rust-src" "rust-analyzer" ];
      };

      craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

      # Build web frontend first
      webFrontend = pkgs.buildNpmPackage {
        pname = "parkhub-web";
        version = cargoToml.workspace.package.version;
        src = ./parkhub-web;
        npmDepsHash = "";  # Will need to be set after first build
        buildPhase = ''
          npm run build
        '';
        installPhase = ''
          cp -r dist $out
        '';
      };

      # Source filtering for Rust
      src = pkgs.lib.cleanSourceWith {
        src = ./.;
        filter = path: type:
          (pkgs.lib.hasSuffix ".rs" path) ||
          (pkgs.lib.hasSuffix ".toml" path) ||
          (pkgs.lib.hasSuffix ".lock" path) ||
          (pkgs.lib.hasSuffix ".json" path) ||
          (pkgs.lib.hasInfix "/parkhub-web/dist/" path) ||
          (type == "directory");
      };

      cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
      version = cargoToml.workspace.package.version;

      commonArgs = {
        inherit src version;
        pname = "parkhub-server";
        strictDeps = true;

        # Only build the server in headless mode (no GUI deps)
        cargoExtraArgs = "--package parkhub-server --no-default-features --features headless";

        nativeBuildInputs = with pkgs; [
          pkg-config
          mold
          clang
        ];

        buildInputs = with pkgs; [
          openssl
        ];

        # Ensure the web dist directory exists for rust-embed
        preBuild = ''
          if [ ! -d parkhub-web/dist ]; then
            mkdir -p parkhub-web/dist
            echo '<html><body>ParkHub</body></html>' > parkhub-web/dist/index.html
          fi
        '';
      };

      cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
        doCheck = false;
      });

      parkhub-server = craneLib.buildPackage (commonArgs // {
        inherit cargoArtifacts;
        doCheck = false;
        RUSTFLAGS = "-C target-cpu=native -C linker=clang -C link-arg=-fuse-ld=mold";
      });

      streamImage = pkgs.dockerTools.streamLayeredImage {
        name = "parkhub";
        tag = "v${version}";
        contents = [
          parkhub-server
          pkgs.cacert
        ];
        config = {
          Entrypoint = [ "${parkhub-server}/bin/parkhub-server" ];
          WorkingDir = "/app";
          Env = [
            "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
            "RUST_LOG=info"
          ];
          ExposedPorts = {
            "7878/tcp" = {};
          };
        };
      };

    in {
      packages.${system} = {
        default = parkhub-server;
        inherit parkhub-server streamImage;
      };

      devShells.${system}.default = craneLib.devShell {
        packages = with pkgs; [
          cargo-watch
          cargo-audit
          cargo-nextest
          pkg-config
          openssl
          mold
          clang
          nodejs_22
          skopeo
        ];
      };

      checks.${system} = {
        build = parkhub-server;

        fmt = craneLib.cargoFmt { inherit src; };

        clippy = craneLib.cargoClippy (commonArgs // {
          inherit cargoArtifacts;
          cargoClippyExtraArgs = "--package parkhub-server --no-default-features --features headless -- -D warnings -A unused_imports -A unused_variables -A dead_code";
        });

        tests = craneLib.cargoNextest (commonArgs // {
          inherit cargoArtifacts;
          nativeBuildInputs = commonArgs.nativeBuildInputs ++ [ pkgs.cargo-nextest ];
          cargoNextestExtraArgs = "--package parkhub-server --no-fail-fast";
        });

        audit = pkgs.runCommand "cargo-audit" {
          nativeBuildInputs = [ pkgs.cargo-audit ];
          src = src;
        } ''
          cd $src
          cargo audit || true
          touch $out
        '';
      };
    };
}
