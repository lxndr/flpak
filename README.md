# Archive utility

A utility to work with some random archive formats.

## Supported formats:

| Format | Description                                                                            | Extension | Extracting | Creating |
| ------ | -------------------------------------------------------------------------------------- | --------- | :--------: | :------: |
| bsa-mw | Bethesda Archive (Morrowind)                                                           | .bsa      |     ✅      |    ✅     |
| bsa    | Bethesda Archive (Oblivion, Fallout 3, New Vegas, Skyrim 2011, Skyrim Special Edition) | .bsa      |     ✅      |    ❌     |
| pak    | id Software PAK                                                                        | .pak      |     ✅      |    ✅     |
| rpa    | Ren'Py Archive                                                                         | .rpa      |     ✅      |    ✅     |
| vpk    | Valve Pak                                                                              | .vpk      |     ✅      |    ❌     |

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

```flpak create --format pak --add-dir ./input_dir --exclude unneeded_file/in_resulting_archive.txt ./archive.ext```

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
