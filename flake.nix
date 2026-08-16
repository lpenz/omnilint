{
  description = "omnilint development environment";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
    in
    {
      devShells = nixpkgs.lib.genAttrs systems (system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              ruff
              shellcheck
              yamllint
              luaPackages.luacheck
              perlPackages.PerlCritic
              clj-kondo
              hadolint
              ktlint
              swiftlint
              sqlfluff
              markdownlint-cli2
              (python3.withPackages (ps: [ ps.flake8 ]))
            ];
          };
        });
    };
}
