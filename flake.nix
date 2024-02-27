{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    geng.url = "github:geng-engine/cargo-geng";
    geng.inputs.nixpkgs.follows = "nixpkgs";
  };
  outputs = { self, geng, nixpkgs, ... }: geng.makeFlakeOutputs (system:
    {
      rust.targets = [
        "aarch64-linux-android"
        "armv7-linux-androideabi"
      ];
      src = geng.lib.${system}.filter {
        root = ./.;
        include = [
          "src"
          "dynamic-deps"
          "assets"
          "Cargo.lock"
          "Cargo.toml"
        ];
      };
      extraBuildInputs =
        let
          pkgs = import nixpkgs {
            inherit system;
          };
        in
        [ pkgs.mold pkgs.clang_14 pkgs.bundletool ];
    });
}
