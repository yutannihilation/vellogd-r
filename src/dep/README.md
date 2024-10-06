This directory will be published as a separated repository, but you can try the
graphics device without an R package.

# Server

You can run server by `cargo run` (I think debug build is enough for testing).

```console
cargo run -p vellogd-server
```

# Client

CLI has subcommands corresponding to each operation.

```console
cargo run -p vellogd-cli -- <COMMAND>
```

For example, you can draw a circle with red fill and green outline at (100, 100).
A color is represented by 3 or 4 hex numbers (RGB or RGBA). `f00`.

```
cargo run -p vellogd-cli -- circle 100 100 --fill f00 --color 0f0 --radius 10
```

Please refer to `--help` for more details.

``` console
A CLI to debug vellogd-server

Usage: vellogd-cli.exe <COMMAND>

Commands:
  close
  clear
  circle
  line
  lines
  polygon
  text
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```