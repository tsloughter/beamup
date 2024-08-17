# BEAMUp

A tool for installing languages that run on the [Erlang
VM](https://www.erlang.org/) (BEAM) on related components.
As of now only Erlang and [Gleam](https://gleam.run/) are supported, with [Elixir](http://elixir-lang.org/)
to come next.

## Install

An install script is provided:

```
$ curl --proto '=https' --tlsv1.2 -LsSf https://github.com/tsloughter/beamup/releases/download/v0.1.1/beamup-installer.sh | sh
```

Binaries can also be downloaded from the [releases on
Github](https://github.com/tsloughter/beamup/releases). Or install from source
using [cargo](https://doc.rust-lang.org/cargo/).

## Usage

`beamup` will store configuration at `~/.config/beamup/config.toml` by default
(no override is yet supported) and local configuration at `./.beamup.toml`.

In `~/.local/bin/` there are binaries created as hard links to the `beamup`
executable for each language command, i.e. `gleam`, `erlc`, `erl`, etc.

Installs are currently done to `~/.cache/beamup/<language>/<id>`. I don't like
that. It doesn't make sense to be under `.cache`. It may be moved under
`~/.local` as well in the next release. Or somewhere else more appropriate by
XDG standards.

### Install

The `build` command will compile a release and `install` will fetch a binary
release. For Erlang at this time only `build` is supported and for Gleam only
`install` is supported`.

The string `latest` can be used instead of a release name to get the release
marked latest in Github:

```
$ beamup build erlang latest
```

```
$ beamup install gleam latest
```

See the `releases <language>` sub-command to see available releases to
build/install.

### Set Default Version

Assuming you've built `OTP-25.3.2.7` you could set the default Erlang to use to
it:

```
$ beamup default erlang OTP-25.3.2.7
```

### Switch Version Used in Directory

Using the `switch` sub-command either appends to or creates `./.beamup.toml`
with an entry like `erlang = "OTP-25.3.2.7"` and running an Erlang command like
`erl` in that directory will use that version instead of the global default.

### Other Commands

- `releases <language>`: List the available releases that can be installed
- `update-links`: Update the hard links that exists for each language executable

## Differences with Erlup

BEAMUp is the successor to [erlup](https://github.com/tsloughter/erlup) and has
important differences. First, the configuration is TOML and not INI, see `
~/.config/beamup/config.toml` and commands require specifying a language to work on,
for example:

```
$ beamup install gleam v1.3.2
```

Another key difference is `build` will work on the tarball of Github releases by
default, not clones of tags. Use `-b` (not supported yet) to install a tag or
branch of a repository.



## Acknowledgments

Inspiration for `erlup` is [erln8](https://github.com/metadave/erln8) by Dave
Parfitt. He no longer maintains it and I figured I could use writing my own as a
way to learn Rust.

The switch to hardlinks instead of symlinks was taken from [rustup](https://rustup.rs/).
