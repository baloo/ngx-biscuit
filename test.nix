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

  biscuit = { datalog, attrs ? { } }: builtins.readFile (
    runCommand "gen-biscuit" { nativeBuildInputs = [ biscuit-cli ]; } ''
            biscuit generate \
              --private-key "${rootkey}" \
      	${lib.cli.toGNUCommandLineShell {} attrs} \
      	${datalog} > $out
    ''
  );

  runTest = { name, authorizer, datalog, makeBiscuitAttrs ? { }, expect ? "succeed" }:
    let
      token = biscuit {
        inherit datalog;
        attrs = makeBiscuitAttrs;
      };
    in
    testers.nixosTest ({
      name = "ngx-biscuit-integration-${name}";

      nodes = {
        webserver = { ... }: {
          services.nginx = {
            enable = true;
            additionalModules = [ nginxModules.biscuit ];

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

        webserver.${expect}(
            "curl --verbose --fail --header 'Authorization: Bearer ${token}' --resolve biscuit:80:127.0.0.1 http://biscuit/"
        )
      '';

    });
in
{
  simple = runTest { name = "simple"; datalog = sample; inherit authorizer; };
  ttl = runTest {
    name = "ttl";
    expect = "fail";
    datalog = sample;
    inherit authorizer;
    makeBiscuitAttrs = {
      add-ttl = "2024-10-07T03:03:01Z";
    };
  };
}
