# Obsidian Property Renamer

A CLI utility for batch renaming properties in Obsidian markdown notes.

This is a small side project to learn rust, don't expect perfect code or any
sort of support!

## Installation

```bash
cargo install rename-obsidian-property
```

## Usage

```sh
Usage: rename-obsidian-property [OPTIONS] <PROPERTY> <NEW_NAME>

Arguments:
  <PROPERTY>
  <NEW_NAME>

Options:
  -p, --pattern <FILE_PATTERN>

  -d, --dry-run
          Run in dry-run mode (don't actually modify files)

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```
