# thematic

A converter between VSCode and Zed color themes. Tested only on MacOS.

## Usage

Converts between VSCode and Zed color theme formats. Supports conversion from VSCode themes to Zed
format and vice versa. Output is written to the destination editor's default theme location.

The commands have aliases: `zv` for zed-to-vscode, and `vz` for vscode to zed. The second positional argument is the name of the theme to convert, or part of the name of the theme. For example, `thematic vz rainglow` finds the "Rainglow" theme extension for VSCode in the standard VSCode extensions directory, converts all included themes to Zed format (grouping into Zed families), then writes everything out as a new Zed extension in the standard Zed extension location.


```text
❯ thematic --help
Usage: thematic [OPTIONS] <COMMAND>

Commands:
  vscode-to-zed  Convert a VSCode theme JSON file to Zed format; `vz` for short
  zed-to-vscode  Convert a Zed theme JSON file to VSCode format; `zv` for short
  help           Print this message or the help of the given subcommand(s)

Options:
  -q						 Print little-to-no theme information
  -v						 Print more theme information
  -h, --help		 Print help (see a summary with '-h')
  -V, --version  Print version
```

```text
❯ thematic vz --help
Convert a VSCode theme JSON file to Zed format; `vz` for short

Usage: thematic vscode-to-zed [OPTIONS] <theme-name>

Arguments:
  <theme-name>  The name or part of the name of a VSCode theme to convert to Zed format

Options:
  -q          Print little-to-no theme information
  -v          Print more theme information
  -h, --help  Print help
```

## Stochastic parrot hints

This was about two-thirds done with some targeted prompting of Claude Sonnet 4, including refactoring when it didn't do a good enough job. The remaining third was me losing patience with telling the intern what to do exactly. Having it do the tedious map-schemas-to-structs part was worth it, as was the From<T> implementations. It was also sort of okay at writing voluminous tests.

My initial input to it was the schemas for both theme formats (in the [schemas directory](./schemas) and examples of each (in the [fixtures directory](./fixures)). I had to do quite a lot of prompting to refactor the results into usability, and it's still pretty messy by even my shoddy standards. However, it did a lot of tedious work for me. When it got going with a solid prompt would slam up against consecutive tool use limits very quickly.

The schema for Zed themes is at [https://zed.dev/schema/themes/v0.2.0.json](https://zed.dev/schema/themes/v0.2.0.json).

The schema for VSCode themes:
* [color-theme schema](https://github.com/wraith13/vscode-schemas/blob/master/en/latest/schemas/color-theme.json) - the overall schema
		- [workbench-colors schema](https://github.com/wraith13/vscode-schemas/blob/master/en/latest/schemas/workbench-colors.json)
		- [textmate-colors schema](https://github.com/wraith13/vscode-schemas/blob/master/en/latest/schemas/textmate-colors.json)

## Developing

Well, there's a file for telling Claude how to work. It also tells humans how to develop. tl;dr Rust. Conveniences in the justfile.

## TODO

- [ ] Convert icon themes.
- [ ] Polish up the user-visible output.

## LICENSE

This code is licensed via [the Parity Public License.](https://paritylicense.com) This license requires people who build on top of this source code to share their work with the commun    ity, too. See the license text for details.
