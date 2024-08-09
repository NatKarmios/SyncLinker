# SyncLinker

A tool that automatically merges & syncs folders via symlinks.

Note that links are only synchronised *at the top level* of each directory; subdirectories will be symlinked as-is, and not recursively explored. If you really want support for recursive linking, feel free to submit an issue (or a PR).

## Usage
```
$ sync_linker --help
Merges & syncs folders via symlinks

Usage: sync_linker [OPTIONS]

Options:
  -q, --quiet            Quiet mode
  -d, --dry-run          Dry run
  -o, --once             Don't watch folders, only run once
      --config <CONFIG>  Config file location [default: ./config.yaml]
  -h, --help             Print help
```

## Example
See `/example`:
```
in1/
  a
in2/
  b
out/
  dead -> in1/dead
```
... becomes ...
```
in1/
  a
in2/
  b
out/
  a -> in1/a
  b -> in2/b
```

## But why?
I made this tool to help with pulling ROMs from multiple sources with [EmuDeck](https://www.emudeck.com/).

