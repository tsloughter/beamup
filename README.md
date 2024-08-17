## BEAMUp

A tool for installing languages that run on the [Erlang
VM](https://www.erlang.org/) (BEAM) on related components.
As of now only Erlang and [Gleam](https://gleam.run/) are supported.

### Install

An install script is provided:

```
...
```

Binaries can also be downloaded from the [releases on
Github](https://github.com/tsloughter/beamup/releases). Or install from source
using [cargo](https://doc.rust-lang.org/cargo/).

### Differences with Erlup

BEAMUp is the successor to [erlup](https://github.com/tsloughter/erlup) and has
important differences. First, the configuration is TOML and not INI, see `
~/.config/beamup/config` and commands require specifying a language to work on,
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
