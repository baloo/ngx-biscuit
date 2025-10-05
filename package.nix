{ rustPlatform, nginx, nix-gitignore, libxcrypt, pcre2, openssl }:

rustPlatform.buildRustPackage {
  pname = "ngx-biscuit";
  version = "0.1.0";

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
  ] ./.;
  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  buildInputs = [
    libxcrypt
    pcre2
    openssl
  ];

  nativeBuildInputs = [
    rustPlatform.bindgenHook
  ];

  # linker issues with tests I haven't figured out
  dontCargoCheck = true;

  NGINX_BUILD_DIR = "${nginx.objs}/objs";
}
