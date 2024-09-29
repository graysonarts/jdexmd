# jdexmd

[Johnny Decimal](https://johnnydecimal.com/10-19-concepts/11-core/11.01-introduction/) is a system for labeling and organizing information.

I've recently started implementing it for my [notes garden](https://garden.graysonarts.com), so I needed a tool to handle documenting the system.

This tool will create both an Obsidian structure as well as a parallel directory structure for non-notes files.

## Usage

```
Usage: jdexmd [OPTIONS] --config-file <CONFIG_FILE>

Options:
  -d, --dry-run                    Preview what actions will be taken
  -c, --config-file <CONFIG_FILE>  The Path to a toml file that defines the system [env: JDEX_CONFIG=example.garden.toml]
  -h, --help                       Print help
  -V, --version                    Print version
	```
