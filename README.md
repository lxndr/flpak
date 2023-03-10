# Archive utility

A utility to work with some random archive formats.

## Supported formats:

| Format | Description                                                                                       | Extension | Extracting | Creating | Params                                                                                              | Comment                        |
| ------ | ------------------------------------------------------------------------------------------------- | --------- | :--------: | :------: | --------------------------------------------------------------------------------------------------- | ------------------------------ |
| bsa-mw | Bethesda Archive (Morrowind)                                                                      | .bsa      |     ✅      |    ✅     |                                                                                                     |
| bsa    | Bethesda Archive (Oblivion, Fallout 3, New Vegas, Skyrim 2011, Skyrim Special Edition, Skyrim VR) | .bsa      |     ✅      |    ✅     | <p> version=103/104/105 <p> compress=true/false <p> xbox=true/false <p> embed-names=true/false |
| ba2    | Bethesda Archive 2 (Fallout 4, Fallout 4 VR, Fallout 76)                                          | .ba2      |     ✅      |    ❌     |                                                                                                     | Only general archive supported |
| pak    | id Software PAK                                                                                   | .pak      |     ✅      |    ✅     |
| rpa    | Ren'Py Archive                                                                                    | .rpa      |     ✅      |    ✅     |
| vpk    | Valve Pak                                                                                         | .vpk      |     ✅      |    ❌     |
| zip    | ZIP                                                                                               | .zip      |     ✅      |    ✅     |

## Usage

```flpak --help```

#### List supported formats

```flpak list-formats```

#### List files

```flpak list ./archive.ext```

#### Check archive correctness

```flpak check ./archive.ext```

#### Extract archive into directory

```flpak extract ./archive.ext ./out```

#### Creating an archive

```flpak create --format pak --add-dir ./input_dir --exclude unneeded_file/in_resulting_archive.txt ./archive.pak```

```flpak create --format bsa --options version=104,compress=true --add-dir ./input_dir --exclude unneeded_file/in_resulting_archive.txt ./archive.bsa```

## Development

#### Build

```
cargo build --release
strip target/release/flpak
```

#### Code coverage

```
cargo tarpaulin --out Html
```
