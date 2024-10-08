{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
        in
        with pkgs;
        {
          formatter = nixpkgs.legacyPackages.x86_64-linux.nixpkgs-fmt;
          devShells.default = mkShell {
            buildInputs = [
              (rust-bin.stable.latest.default.override {
                extensions = [ "rust-src" ];
              })
              rust-bin.stable.latest.default
              pkg-config

              rust-analyzer
            ];

            nativeBuildInputs = [
              pkg-config
            ];

            RUSTFLAGS = map (a: "-C link-arg=${a}") [
              "-Wl,--push-state,--no-as-needed"
              "-Wl,--pop-state"
            ];

            LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];

            # LD_LIBRARY_PATH = lib.makeLibraryPath [
            #   libxkbcommon
            #   mesa.drivers
            #   vulkan-loader
            #   xorg.libX11
            #   xorg.libXcursor
            #   xorg.libXi
            #   xorg.libXrandr
            # ];
          };
        }
      );
}
