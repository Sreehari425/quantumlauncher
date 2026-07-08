{
  description = "QuantumLauncher";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      forAllSystems =
        f:
        nixpkgs.lib.genAttrs systems (
          system:
          f (
            import nixpkgs {
              inherit system;
              overlays = [ (import rust-overlay) ];
            }
          )
        );

      cargoToml = builtins.fromTOML (builtins.readFile ./quantum_launcher/Cargo.toml);

      commonAttrs = pkgs: {
        pname = cargoToml.package.name;
        version = cargoToml.package.version;

        cargoLock = {
          lockFile = ./Cargo.lock;
          outputHashes = {
            "dark-light-2.0.0" = "sha256-4JPfNSk8MsMMxJPz6M8ju+sMVXlAerct16xwbkEWpYw=";
            "http-cache-1.0.0-alpha.6" = "sha256-ZnrX0bnLARjwYeH4YnhwidAk2st124rGCR40oyFun4U=";
            "http-cache-semantics-3.0.0" = "sha256-D2n73HWEnhaBEZOwTzAOQ6kd2bO9oin5vEZ+zangJ9A=";
          };
        };

        buildAndTestSubdir = "quantum_launcher";

        nativeBuildInputs = with pkgs; [
          pkg-config
          makeWrapper
        ];

        buildInputs = with pkgs; [
          libGL
          libxkbcommon
          vulkan-loader
          wayland
          wayland-protocols
          libx11
          libxcursor
          libxi
          libxrandr
        ];

        postInstall = ''
          wrapProgram $out/bin/quantum_launcher \
            --prefix LD_LIBRARY_PATH : "${
              with pkgs;
              nixpkgs.lib.makeLibraryPath [
                wayland
                libxkbcommon
                libGL
                vulkan-loader
              ]
            }"
        '';

        meta = {
          mainProgram = "quantum_launcher";
        };
      };
    in
    {
      # ==========================================
      # 1. PACKAGES (For nix run, build, and tags)
      # ==========================================
      packages = forAllSystems (pkgs: {
        default = pkgs.rustPlatform.buildRustPackage (
          (commonAttrs pkgs)
          // {
            src = ./.;
          }
        );

        release = pkgs.rustPlatform.buildRustPackage (
          (commonAttrs pkgs)
          // {
            src = pkgs.lib.cleanSource ./.;
          }
        );
      });

      # ==========================================
      # 2. DEVELOPMENT ENVIRONMENT (nix develop)
      # ==========================================
      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          nativeBuildInputs = [
            (pkgs.rust-bin.stable."1.85.0".default.override {
              extensions = [
                "rust-src"
                "rust-analyzer"
              ];
            })
            pkgs.pkg-config
          ];

          buildInputs = (commonAttrs pkgs).buildInputs;

          LD_LIBRARY_PATH = nixpkgs.lib.makeLibraryPath (
            with pkgs;
            [
              wayland
              libxkbcommon
              libGL
              vulkan-loader
            ]
          );
        };
      });

      # ==========================================
      # 3. NIXOS MODULE (System-wide configuration)
      # ==========================================
      nixosModules.default =
        {
          config,
          lib,
          pkgs,
          ...
        }:
        let
          cfg = config.programs.quantum-launcher;
        in
        {
          options.programs.quantum-launcher = {
            enable = lib.mkEnableOption "QuantumLauncher game client";
            package = lib.mkOption {
              type = lib.types.package;
              default = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
              description = "The QuantumLauncher package to install.";
            };
          };

          config = lib.mkIf cfg.enable {
            environment.systemPackages = [ cfg.package ];
          };
        };

      # ==========================================
      # 4. HOME MANAGER MODULE (User-level configs)
      # ==========================================
      homeManagerModules.default =
        {
          config,
          lib,
          pkgs,
          ...
        }:
        let
          cfg = config.programs.quantum-launcher;
        in
        {
          options.programs.quantum-launcher = {
            enable = lib.mkEnableOption "QuantumLauncher game client via Home Manager";
            package = lib.mkOption {
              type = lib.types.package;
              default = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
              description = "The QuantumLauncher package to install.";
            };
          };

          config = lib.mkIf cfg.enable {
            home.packages = [ cfg.package ];
          };
        };
    };
}
