{
    inputs = {
        nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
        naersk.url = "github:nix-community/naersk";
    };

    outputs = { self, nixpkgs, naersk }:
    let
        supportedSystems = [
          "x86_64-linux"
          "aarch64-linux"
          "i686-linux"
        ];
        forAllSystems = f: builtins.listToAttrs (map (system: {
          name = system;
          value = f system;
        }) supportedSystems);
    in {
        packages = forAllSystems (system:
         let
            pkgs = import nixpkgs { inherit system; };
            naerskLib = pkgs.callPackage naersk {};
         in {
            default = naerskLib.buildPackage {
                src = ./.;
                buildInputs = with pkgs; [ glib pciutils usbutils cpuid ];
                nativeBuildInputs = with pkgs; [ pkg-config ];
            };
            debug = naerskLib.buildPackage {
                src = ./.;
                release = false;
                buildInputs = with pkgs; [ glib pciutils usbutils cpuid ];
                nativeBuildInputs = [ pkgs.pkg-config ];
            };
         });
    };
}
