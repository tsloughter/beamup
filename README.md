# BEAMUp

[![Release](https://github.com/tsloughter/beamup/actions/workflows/release.yml/badge.svg)](https://github.com/tsloughter/beamup/actions/workflows/release.yml)

<p align="center">
    <img alt="beamup logo" src="https://github.com/user-attachments/assets/b1d7c5da-71f1-4c15-96fe-a01e3884d523">
</p>

A tool for installing languages (support for Gleam, Erlang and Elixir) that run
on the [Erlang VM](https://www.erlang.org/) (BEAM) and related components --
including ELP and rebar3.

## Install

An install script is provided for both Linux/Mac:

```
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/tsloughter/beamup/releases/download/v0.10.0/beamup-installer.sh | sh
```

And Windows Powershell:

```
powershell -c "irm
https://github.com/tsloughter/beamup/releases/download/v0.8.0/beamup-installer.ps1
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

Installs are stored at `<install_dir>/<language-or-component>/<install-id>`.
The `install_dir` setting in the global config controls the root for installing,
building, listing, and running tools. Its default is the application's data
directory, giving these paths:

- Linux: `~/.local/share/beamup/<language>/<id>`
- Mac: `~/Library/Application Support/beamup/<language>/<id>`
- Windows: `~\AppData\Local\beamup\<language>\<id>`

For example, a global config can contain:

```toml
install_dir = "/srv/beamup"

[erlang]
default = "otp-debug"
default_build_options = "--enable-lock-counter"

[rebar3]
default = "3.23.0"
```

This selects `/srv/beamup/erlang/otp-debug` and
`/srv/beamup/rebar3/3.23.0`. A relative `install_dir` is resolved relative to
the global config file's directory. Project `.beamup.toml` files select install
IDs within this same root.

There is no install registry in the config. `beamup list` discovers language
and component installs from directories containing `bin`, and sorts their IDs.
Existing directories with this layout can be used without metadata. New installs
contain a `.beamup-install.toml` file recording the requested release, the OTP
major version used to select an Elixir or ELP asset, and build options for source builds,
where applicable. Metadata is descriptive; it is not used to locate an install.

An install ID must be a single directory name. Absolute paths, `.` and `..`,
path separators, control characters, Windows-invalid filename characters, and
trailing dots or spaces are rejected. The `.beamup-install-` prefix is reserved
for staging directories. If a release or branch name contains a slash, supply a
separate ID with `--id`.

Archive installs and source builds are prepared in temporary directories on the
same filesystem as the destination. They are published after success, with
metadata already in place. Windows native installers run at the final path so
that OS registrations use the correct directory; an incomplete marker hides
their output until success. Discovery ignores staging directories and installs
marked incomplete. Failed installs and builds restore or preserve the previous
install when `--force` is used.
Erlang source builds use the final configured prefix with `DESTDIR` staging;
install path configure options such as `--prefix` are managed by beamup.

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
release. At this time Gleam and Elixir only the `install` command is supported`.

Release names are tried as exact tags first. If no release matches, beamup tries
adding `OTP-` for Erlang or `v` for Elixir and Gleam. For example,
`beamup build erlang 28.0` can fetch `OTP-28.0`, and `beamup install gleam 1.10.0`
can fetch `v1.10.0`. The same fallback applies to both `build` and `install`.

Erlang installs are static builds from
[gleam-community/erlang-linux-builds](https://github.com/gleam-community/erlang-linux-builds).
To install a build that dynamically links against libc so that NIFs work use the
argument `--static glibc` or `--static musl`.

The string `latest` can be used instead of a release name to get the release
marked latest in Github:

```
$ beamup install erlang latest
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

- `list`: List installed languages and components from the filesystem
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
