# find-images

Find images (sort by last modified by default).
Useful for piping into sxiv/nsxiv when hidden subfiles/directories are significant.

```
Hardlink duplicate files recursively

This tool should only be used when you are sure that duplicate files should remain duplicate in perpetuity

Usage: lndups [OPTIONS] [TARGET]...

Arguments:
  [TARGET]...  Target files and directories (recursive)
                 Each SEPARATOR denotes a new set of targets
                   Each set of targets are separate from all other sets
                   All targets in a set must be on the same device
                 Symlinks are ignored
                 '-' is not treated as special
                 Mutually exclusive with --target-file

Options:
  -v, --verbose...             Increase verbosity
  -q, --quiet...               Decrease verbosity
  -r, --raw-output             Show only hardlink operations and errors, in an easily parseable format
                                 Outputs two columns separated by a tab
                                 Bypasses verbosity
  -n, --no-brace-output        Disable brace notation for output
                                 Ex: /home/user/{dir,backup}/file
  -d, --dry-run                Perform no operations on the filesystem
  -i, --prompt                 Prompt once before operating
                                 Doesn't occurs if no targets are provided
  -m, --min-size <SIZE>        Minimum file size to be considered for hardlinking
                                 Never goes below 1 [default: 1]
  -t, --threads <NUMBER>       Number of threads [default: 2]
  -s, --separator <SEPARATOR>  Separator between sets of targets [default: ;]
  -f, --target-file <FILE>     File to source targets from (can be '-' for stdin)
                                 Same rules as CLI argument targets apply
                                 Mutually exclusive with CLI argument targets
  -h, --help                   Print help
```



### Find non-images
Obtain non-image files by giving any extensions to `--extensions`





## Install


### Cargo package
```bash
cargo install find-images
```


### Debian package
Debian packages are available for stable and oldstable releases.

#### Install the signing key
Clearnet:
```bash
curl https://deb.ascendforever.com/ascendforever.gpg | sudo tee /usr/share/keyrings/ascendforever.gpg >/dev/null
```
Or onion:
```bash
curl http://csjkrevghycpr6b266bk2hrgfotoxsz7xbyfk6rkk63fxlbkbes7b7qd.onion | sudo tee /usr/share/keyrings/ascendforever.gpg >/dev/null
```

#### Add repository
Change `trixie` -> `bookworm` if needed.

Clearnet:
```bash
printf 'deb [signed-by=/usr/share/keyrings/ascendforever.gpg] https://deb.ascendforever.com %s main' trixie | sudo tee /etc/apt/sources.list.d/ascendforever.list
```
Or onion:
```bash
printf 'deb [signed-by=/usr/share/keyrings/ascendforever.gpg] tor+http://csjkrevghycpr6b266bk2hrgfotoxsz7xbyfk6rkk63fxlbkbes7b7qd.onion %s main' trixie | sudo tee /etc/apt/sources.list.d/ascendforever.list
```

#### Install
```bash
sudo apt install -y find-images
```
