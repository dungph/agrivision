{
  inputs = {
    nixpkgs.url = "github:NixOs/nixpkgs/nixos-24.05";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };
  outputs = {self, nixpkgs, flake-utils, rust-overlay}: 
    flake-utils.lib.eachDefaultSystem 
    (system: 
      let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
          crossSystem = {
            config = "aarch64-unknown-linux-musl";
          };
        };
        #toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        
      in 
      {
        #packages = {
        #  default = derivation {
        #    name = "simple";
        #    src = "./.";
        #    builder = "${bash}/bin/bash";
        #    args = [ "-c" "echo foo > $out" ];
        #  };
        #};
        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.rust-bin.stable.latest.default 
            ##toolchain
            pkgs.gcc
          ];
        };
      }
    );
}
