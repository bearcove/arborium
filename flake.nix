{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      forAllSystems = nixpkgs.lib.genAttrs nixpkgs.lib.systems.flakeExposed;
    in
    {
      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs {
            inherit system;
          };
        in
        {
          default = pkgs.mkShell {
            buildInputs = with pkgs; [
              binaryen
              cargo-nextest
              jq
              pnpm
              rustup
              tree-sitter
              wasm-bindgen-cli
              wasm-pack
              wild-unwrapped
            ];
            shellHook = ''
              rustup toolchain install stable
              rustup target add wasm32-unknown-unknown
              rustup component add clippy rustfmt
              rustup toolchain install nightly
              rustup target add wasm32-unknown-unknown --toolchain nightly
              rustup component add rust-src --toolchain nightly
            '';

            CARGO_TARGET_X86_64_LINUX_UNKNOWN_GNU_RUSTFLAGS = "-C link-arg=--ld-path=${pkgs.wild}/bin/wild";
            CARGO_TARGET_X86_64_LINUX_UNKNOWN_GNU_LINKER = "clang";
          };
        }
      );
    };
}
