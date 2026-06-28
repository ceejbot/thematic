# thematic

A bidirectional converter between VSCode and Zed color and icon themes. Works on macOS, Linux, and Windows.

## Installation

```shell
cargo install --path .
```

## Usage

Converts between VSCode and Zed theme formats, including both color themes and icon themes. Output is written to the destination editor's default theme location.

The commands have aliases: `vz` for vscode-to-zed, and `zv` for zed-to-vscode. The search commands also have aliases: `vsc` for vs-code-list and `zed` for zed-list. The positional argument is the name (or part of the name) of the theme to convert or search for.

For example, `thematic vz catppuccin` finds the Catppuccin theme extension in the standard VSCode extensions directory, converts all included color themes and icon themes to Zed format (grouping into Zed families), then writes everything out as a new Zed extension in the standard Zed extension location.

```text
❯ thematic --help
Usage: thematic [OPTIONS] <COMMAND>

Commands:
  vscode-to-zed  Convert a VSCode theme JSON file to Zed format; `vz` for short
  zed-to-vscode  Convert a Zed theme JSON file to VSCode format; `zv` for short
  zed-list       Find all the Zed themes with names matching the input pattern; `zed` for short
  vs-code-list   Find all the VSCode themes with names matching the input pattern; `vsc` for short
  help           Print this message or the help of the given subcommand(s)

Options:
  -q             Quiet output
  -v             Output with theme information
      --dry-run  Show what would be converted without writing any files
  -h, --help     Print help (see a summary with '-h')
  -V, --version  Print version
```

## Stochastic parrot hints

This was about two-thirds done with some targeted prompting of Claude Sonnet 4, including refactoring when it didn't do a good enough job. The remaining third was me losing patience with telling the intern what to do exactly. Having it do the tedious map-schemas-to-structs part was worth it, as was the From<T> implementations. It was also sort of okay at writing voluminous tests. When it got going with a solid prompt would slam up against consecutive tool use limits very quickly.

My initial input to it was the schemas for both theme formats (in the [schemas directory](./docs/schemas) and examples of each (in the [fixtures directory](./fixtures)). I had to do quite a lot of prompting to refactor the results into usability, and it's still pretty messy by even my shoddy standards.

The schema for Zed themes is at [https://zed.dev/schema/themes/v0.2.0.json](https://zed.dev/schema/themes/v0.2.0.json).

The schema for VSCode themes:

* [color-theme schema](https://github.com/wraith13/vscode-schemas/blob/master/en/latest/schemas/color-theme.json)
* the overall schema
	- [workbench-colors schema](https://github.com/wraith13/vscode-schemas/blob/master/en/latest/schemas/workbench-colors.json)
	- [textmate-colors schema](https://github.com/wraith13/vscode-schemas/blob/master/en/latest/schemas/textmate-colors.json)

## Developing

Well, there's a file for telling Claude how to work. It also tells humans how to develop. tl;dr Rust. Conveniences in the justfile.

## TODO

- [ ] Handle some parts of Zed themes that VSCode doesn't do.
- [x] Parse VSCode `fontStyle` combinations (e.g. "bold italic"). `italic`/`oblique` map to Zed's `font_style` and `bold` to `font_weight`, in both directions. `underline`/`strikethrough` are dropped because Zed's theme format has no field for them.
- [x] Convert icon themes (including file copying).
- [x] Polish up the user-visible output.
- [x] Clean up code architecture.
- [x] Detect Linux and use appropriate paths.
- [x] Windows paths implemented.

## LICENSE

This code is licensed via [the Parity Public License.](https://paritylicense.com) This license requires people who build on top of this source code to share their work with the community, too. See the license text for details.
