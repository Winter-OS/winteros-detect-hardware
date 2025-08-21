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
                buildInputs = with pkgs; [ glib ];
                runtimeInputs = with pkgs; [ pciutils usbutils cpuid ];
                nativeBuildInputs = with pkgs; [ pkg-config makeWrapper ];

                postInstall = ''
                    wrapProgram $out/bin/winteros-detect-hardware \
                        --prefix PATH : ${pkgs.pciutils}/bin \
                        --prefix PATH : ${pkgs.usbutils}/bin \
                        --prefix PATH : ${pkgs.cpuid}/bin
                  '';
            };
            debug = naerskLib.buildPackage {
                src = ./.;
                release = false;
                buildInputs = with pkgs; [ glib ];
                runtimeInputs = with pkgs; [ pciutils usbutils cpuid ];
                nativeBuildInputs = with pkgs; [ pkg-config makeWrapper ];

                postInstall = ''
                wrapProgram $out/bin/winteros-detect-hardware \
                    --prefix PATH : ${pkgs.pciutils}/bin \
                    --prefix PATH : ${pkgs.usbutils}/bin \
                    --prefix PATH : ${pkgs.cpuid}/bin
                '';
            };
         });
    };
}
