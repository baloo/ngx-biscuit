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
