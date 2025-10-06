# Nginx Biscuit authentication module

This is a module for nginx written in Rust to authenticate requests via a Biscuit bearer token.

```
load_module "./target/debug/libngx_biscuit.so";

http {
    server {
        location / {
	        auth_biscuit_public_key "ed25519/0a843f91366c1b17bed9715ace06beed1ffb64b3ae755f9d20e94a5b29f5bf68";
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
$ curl -v --header 'Authorization: Bearer En0KEwoEMTIzNBgDIgkKBwgKEgMYgAgSJAgAEiCnqxPdhwS5eVzgmLfNERSe39tXuO0PsPm9KPdQ37qzyBpA3EiRG9764PZRyeirjpX8Hjh4nvEh7YA9YDBY4L3bxeNRTEC-zVHDPOg_JoSQrXuQ4mgIRHRLljvNQdgLgEyQBSIiCiAKpGDK-8sPHTP3XNOwmj5yQswLSEhdfRuHcr5AOrXTQA==' http://localhost:8080
```
