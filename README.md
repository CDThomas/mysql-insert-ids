Dependencies:
* Rust: https://www.rust-lang.org/tools/install
* `sqlx-cli`: install with `cargo install sqlx-cli`
* Docker
* Direnv (if using .envrc file)

Running:
```
$ cp .envrc.template .envrc
$ direnv allow
$ docker-compose up -d
$ sqlx migrate run
$ cargo run
```
