{
  description = "QuantumLauncher";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f (import nixpkgs { inherit system; }));
    in
    {
      packages = forAllSystems (pkgs: {
        default = pkgs.rustPlatform.buildRustPackage {
          pname = "quantum-launcher";
          version = "0.5.2";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
              "dark-light-2.0.0" = "sha256-4JPfNSk8MsMMxJPz6M8ju+sMVXlAerct16xwbkEWpYw=";
              "http-cache-1.0.0-alpha.6" = "sha256-ZnrX0bnLARjwYeH4YnhwidAk2st124rGCR40oyFun4U=";
              "http-cache-semantics-3.0.0" = "sha256-D2n73HWEnhaBEZOwTzAOQ6kd2bO9oin5vEZ+zangJ9A=";
            };
          };

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
      });
    };
}
