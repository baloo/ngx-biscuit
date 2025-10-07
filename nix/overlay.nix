self: super: {
  nginx-biscuit = self.nginxStable.override {
    modules = self.lib.unique (
      self.nginxStable.modules ++ [
        self.nginxModules.biscuit
      ]
    );
  };

  nginxModules = super.nginxModules // {
    biscuit = let
      inherit (self) openssl cargo nix-gitignore rustPlatform rustc pkg-config runCommand ;
    in rec {
      name = "biscuit";
      src =
        let
          src = nix-gitignore.gitignoreSource [
            "authorizer"
            "flake.lock"
            "flake.nix"
            "html"
            "package.nix"
            "README.md"
            "shell.nix"
            "test.nix"
            "tmp"
            "example.cnf"
            ".jj"
          ] ../.;

          combined =
            runCommand "vendored-repo"
              {
                nativeBuildInputs = [
                  rustPlatform.cargoSetupHook
                ];
                cargoDeps = rustPlatform.importCargoLock {
                  lockFile = "${src}/Cargo.lock";
                };
              }
              ''
                mkdir -p $out
                cp -r ${src}/* $out/

                runHook postUnpack
                cp -r cargo-vendor-dir $out/
                cp -r .cargo $out/
              '';
        in
        combined;

      preConfigure = ''
        export NGX_RUSTC_OPT="--config ${src}/.cargo/config.toml"
      #  export OPENSSL_NO_VENDOR=1
      '';

      inputs = [
        openssl
        cargo
        rustPlatform.bindgenHook
        rustc
        pkg-config
      ];

      meta = with self.lib; {
        description = "An NGINX module providing authentication with biscuit tokens";
        homepage = "https://github.com/baloo/ngx-biscuit";
        license = with licenses; [ asl20 ];
        maintainers = with maintainers; [ baloo ];
      };
    };
  };
}
