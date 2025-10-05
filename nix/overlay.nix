self: super: {
  nginx = super.nginx.overrideAttrs (old: {
    outputs = old.outputs ++ [
      "objs"
    ];

    disallowedReferences = [ ];

    postInstall = ''
      mkdir $objs
      cp -a src objs $objs/
    '';
  });

  ngx-biscuit = self.callPackage ../package.nix {};
}
