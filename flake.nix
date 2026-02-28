{
  description = "Overlays for simracing in rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
      
      # Grouped runtime dependencies so they can be easily shared
      runtimeLibs = with pkgs; [
        wayland
        libxkbcommon
        vulkan-loader
      ];
    in
      {
      devShells.${system}.default = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
          rustc
          cargo
          cargo-watch
          clippy
          pkg-config
          valgrind
        ];

        # buildInputs is the semantic place for libraries
        buildInputs = runtimeLibs;

        shellHook = ''
          export WINIT_UNIX_BACKEND=wayland
          export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath runtimeLibs}:$LD_LIBRARY_PATH
        '';
      };

      packages.${system}.default = pkgs.rustPlatform.buildRustPackage {
        pname = "simtem";
        version = "0.1.0";
        src = ./.;

        cargoLock.lockFile = "${self}/Cargo.lock";

        # Added makeWrapper to nativeBuildInputs
        nativeBuildInputs = [ pkgs.pkg-config pkgs.makeWrapper ];
        buildInputs = runtimeLibs;

        WINIT_UNIX_BACKEND = "wayland";

        # Wrap the binary post-install to guarantee it can find the libraries
        postInstall = ''
          wrapProgram $out/bin/simtem \
            --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath runtimeLibs}"
        '';
      };
    };
}
