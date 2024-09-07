# BEAMUp

[![Release](https://github.com/tsloughter/beamup/actions/workflows/release.yml/badge.svg)](https://github.com/tsloughter/beamup/actions/workflows/release.yml)

<p align="center">
    <img alt="beamup logo" src="https://github.com/user-attachments/assets/b1d7c5da-71f1-4c15-96fe-a01e3884d523">
</p>

A tool for installing languages (support for Gleam, Erlang and Elixir) that run
on the [Erlang VM](https://www.erlang.org/) (BEAM) and related components --
component support to come in the future.

## Install

An install script is provided for both Linux/Mac:

```
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/tsloughter/beamup/releases/download/v0.4.7/beamup-installer.sh | sh
```

And Windows Powershell:

```
powershell -c "irm
https://github.com/tsloughter/beamup/releases/download/v0.4.7/beamup-installer.ps1
| iex"
```

Binaries can also be downloaded from the [releases on
Github](https://github.com/tsloughter/beamup/releases). Or install from source
using [cargo](https://doc.rust-lang.org/cargo/).

## Usage

`beamup` will store configuration at:

- Linux: `~/.config/beamup/config.toml` 
- Mac: `~/Library/Application Support/beamup/config.toml`
- Windows: `~\AppData\Local\beamup\config.toml`

Local configuration to set a language/component to use in a specific directory
is in `./.beamup.toml`.

Hard links to the `beamup` executable for each language command, i.e. `gleam`,
`erlc`, `erl`, `iex`, etc, is created in the following directory:

- Linux: `$XDG_BIN_HOME` or `$XDG_DATA_HOME/../bin` or `$HOME/.local/bin`
- Mac: `~/.beamup/bin`
- Windows: `~\.beamup\bin`

This directory must be added to your `PATH` for `beamup` installs to work.

Installs are currently done to the applications data directory. This defaults
to:

- Linux: `~/.local/share/beamup/<language>/<id>`
- Mac: `~/Library/Application Support/beamup/<language>/<id>`
- Windows: `~\AppData\Local\beamup\<language>\<id>`

For languages that support building from source you can pass additional build
options (like what is passed to `./configure` for Erlang) with either the
environment variable `BEAMUP_BUILD_OPTIONS` or adding `default_build_options` to
the configuration under the language section:

```
[erlang]
default_build_options = "--enable-lock-counter"
```

Or:

```
BEAMUP_BUILD_OPTIONS="--enable-lock-counter" beamup build erlang -i latest-lock-counter latest
```

### Install Languages

The `build` command will compile a release and `install` will fetch a binary
release. For Erlang at this time only `build` is supported and for Gleam and
Elixir only `install` is supported`.

The string `latest` can be used instead of a release name to get the release
marked latest in Github:

```
$ beamup build erlang latest
```

```
$ beamup install gleam latest
```

```
$ beamup install elixir latest
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

### Install Components

The `component install` command can install binary releases of tools, currently
[The Erlang Language
Platform](https://whatsapp.github.io/erlang-language-platform/) and
[rebar3](https://rebar3.org/).

The same as with a language you can specify a version of the component to use in
the `.beamup.toml` file in a directory:

```
rebar3 = "3.23.0"
```

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
