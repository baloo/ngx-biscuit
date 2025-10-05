with import ./nix;

let
  vi = neovim.override {
    configure = {
      customRC = ''
        set mouse=
        let g:rustfmt_autosave = 1
      '';

      packages.rust = with vimPlugins; {
        start = [ rust-vim ];
        opt = [ ];
      };
    };
  };
in
mkShell {
  nativeBuildInputs = [
    cargo
    rustc
    rustfmt
    clippy

    biscuit-cli

    vi
    nixpkgs-fmt

    openssl
    pkg-config
    rustPlatform.bindgenHook

    nginx
  ];

  shellHook = ''
    alias vi="${vi}/bin/nvim";
    alias vim=vi;
  '';

  buildInputs = [
    openssl
  ];

  NIX_CFLAGS_COMPILE = "-I${libxcrypt}/include -I${pcre2.dev}/include";
  #NIX_CFLAGS_COMPILE="-I${openssl.dev}/include";
  OPENSSL_INCLUDE_DIR = "${openssl.dev}/include";
  EDITOR = "vi";

  NGINX_BUILD_DIR = "${nginx.objs}/objs";
}
