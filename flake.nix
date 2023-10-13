{
  description = "Dev env";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, fenix }: flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        inherit system;
      };
      buildInputs = with pkgs; [
        fenix.packages.${system}.rust-analyzer
        (fenix.packages.${system}.complete.withComponents [
          "cargo"
          "clippy"
          "rust-src"
          "rustc"
          "rustfmt"
        ])
        openssl
        pkg-config
      ];
    in
    {
      # `eachDefaultSystem` transforms the input, our output set
      # now simply has `packages.default` which gets turned into
      # `packages.${system}.default` (for each system)
      devShells.default = pkgs.mkShell {
        inherit buildInputs;
      };
    }
  );
}
