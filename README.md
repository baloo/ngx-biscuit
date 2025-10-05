# Nginx Biscuit authentication module

This is a module for nginx written in Rust to authenticate requests via a Biscuit bearer token.

```
http {
    server {
        location / {
	        auth_biscuit_public_key "ed25519/7fb375e762b525d926755ddf0dc68e413dce33c46f065d9b9423a827f2c3df0e";
	        auth_biscuit_authorizer_file /path/to/authorizer;
        }
    }
}
```


## Development

```
$ nix-shell
$ cargo build
$ nginx -c example.cnf -e stderr -p $PWD
$ curl -v --header 'Authorization: Bearer En0KEwoEMTIzNBgDIgkKBwgKEgMYgAgSJAgAEiAdbUPkSZ4KVak-CIGWs_shxjAync_e13qnaHD6Am3DSRpArECNcshzWMj22A5AQySpNIpJ3zlQNgfCU16pqg2V6N8Yw5CfFqUvo8qrG9qC3-M3PPdadYt-xrG2ETRWZxsnBCIiCiCZB1G2yrztVqbYO4giPTLaoSDGtdwypEUbP5W2DvT4Hg==' http://localhost:8080
```
