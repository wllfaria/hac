{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = {
    nixpkgs,
    rust-overlay,
    crane,
    ...
  }: let
    inherit (nixpkgs) lib;
    withSystem = f:
      lib.fold lib.recursiveUpdate {}
      (map f ["x86_64-linux" "x86_64-darwin" "aarch64-linux" "aarch64-darwin"]);
  in
    withSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [rust-overlay.overlays.default];
        };
        inherit (pkgs) stdenv lib;
        toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
        buildDeps =
          []
          ++ lib.optionals stdenv.isDarwin (with pkgs.darwin.apple_sdk.frameworks; [
            SystemConfiguration
          ]);
        crate = craneLib.buildPackage {
          # Filter src to only .rs, .toml, and .scm files to avoid needless recompiles
          src = let
            schemeFilter = path: _type: builtins.match ".*scm$" path != null;
            schemeOrCargo = path: type:
              (schemeFilter path type) || (craneLib.filterCargoSources path type);
          in
            lib.cleanSourceWith {
              src = craneLib.path ./.;
              filter = schemeOrCargo;
            };
          strictDeps = true;
          buildInputs = buildDeps;
          nativeBuildInputs = lib.optionals stdenv.isLinux [pkgs.pkg-config];
          doNotSign = stdenv.isDarwin; # Build seems to break when this is true on Darwin
        };
      in {
        apps.${system}.default = let
          name = crate.pname or crate.name;
          exe = crate.passthru.exePath or "/bin/${name}";
        in {
          type = "app";
          program = "${crate}${exe}";
        };
        packages.${system}.default = crate;
        checks.${system} = {inherit crate;};
        devShells.${system}.default = pkgs.mkShell {
          packages = with pkgs;
            [
              toolchain
              rust-analyzer-unwrapped
            ]
            ++ buildDeps;
          RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
        };
      }
    );
}
