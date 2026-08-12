{
  description = "mdpdf - Markdown-to-PDF transducer";

  nixConfig = {
    extra-substituters = [ "https://flowerornament.cachix.org" ];
    extra-trusted-public-keys = [
      "flowerornament.cachix.org-1:gSODgIXgfRANrEGITBOF8XWaEKNy8hkNGfRVwqUG46c="
    ];
  };

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      forAllSystems = nixpkgs.lib.genAttrs [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "mdpdf";
            version = "0.3.1";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            # `just check` owns tests; rebuilding LTO test binaries here only
            # duplicates the release gate and can exhaust local Nix storage.
            doCheck = false;
            meta.mainProgram = "mdpdf";
          };
        });
      apps = forAllSystems (system: {
        default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/mdpdf";
        };
      });
    };
}
