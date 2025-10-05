with import ./nix;

let
  rootkey = "ed25519-private/63e66593943970af8520ed0e6424ed791823a412728475268f6e88b993bb50b6";
  public = runCommand "public" { nativeBuildInputs = [ biscuit-cli ]; } ''
    biscuit keypair --from-private-key "${rootkey}" | grep Public | cut -f 3 -d ' ' > $out
  '';
  authorizer = writeText "authorizer" ''
    allow if user($u);
  '';

  sample = writeText "sample" ''
    user("1234");
  '';

  biscuit = datalog: builtins.readFile (
    runCommand "gen-biscuit" { nativeBuildInputs = [ biscuit-cli ]; } ''
      biscuit generate --private-key "${rootkey}" ${datalog} > $out
    ''
  );
in
testers.nixosTest ({
  name = "ngx-biscuit-integration";

  nodes = {
    webserver = { ... }: {
      services.nginx = {
        enable = true;
        prependConfig = ''
            load_module "${ngx-biscuit}/lib/libngx_biscuit.so";
        '';
        virtualHosts.biscuit = {
	  locations."/" = {
            extraConfig = ''
              auth_biscuit_public_key "${builtins.readFile public}";
              auth_biscuit_authorizer_file ${authorizer};
            '';
	  };
        };
      };
    };
  };

  testScript = ''
    webserver.wait_for_unit("nginx")
    webserver.wait_for_open_port(80)

    webserver.fail(
        "curl --verbose --fail --resolve biscuit:80:127.0.0.1 http://biscuit/"
    )

    webserver.succeed(
        "curl --verbose --fail --header 'Authorization: Bearer ${biscuit sample}' --resolve biscuit:80:127.0.0.1 http://biscuit/"
    )
  '';

})
