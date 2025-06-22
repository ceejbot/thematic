# thematic

A converter between vs code and zed color themes. Not quite finished, but nearly there.

About 90% done with some targeted prompting of Claude Sonnet 4, including refactoring when it didn't do a good enough job. The remaining 10% was me losing patience with telling the intern what to do exactly. Having it do the tedious map-schemas-to-structs part was worth it, as was the From<T> implementations. The rest? Meh.

The schema for Zed themes is at [https://zed.dev/schema/themes/v0.2.0.json](https://zed.dev/schema/themes/v0.2.0.json).

The schema for VSCode themes:
* [color-theme schema](https://github.com/wraith13/vscode-schemas/blob/master/en/latest/schemas/color-theme.json) - the overall schema
		- [workbench-colors schema](https://github.com/wraith13/vscode-schemas/blob/master/en/latest/schemas/workbench-colors.json)
		- [textmate-colors schema](https://github.com/wraith13/vscode-schemas/blob/master/en/latest/schemas/textmate-colors.json)

## Developing

Well, there's a file for telling Claude how to work. It also tells humans how to develop. tl;dr Rust. Conveniences in the justfile.

## LICENSE

This code is licensed via [the Parity Public License.](https://paritylicense.com) This license requires people who build on top of this source code to share their work with the commun    ity, too. See the license text for details.
