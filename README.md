# Archive utility

A utility to work with some random archive formats.

## Supported formats:

Format | Description                | Extension | Extracting | Creating
------ | -------------------------- | --------- | :--------: | :--------:
bsa    | Bethesda Softworks Archive | .bsa      | ✅         | ✅
bsa2   | Bethesda Softworks Archive | .bsa      | ✅         | ❌
pak    | id Software PAK            | .pak      | ✅         | ✅
rpa    | Ren'Py Archive             | .rpa      | ✅         | ✅
vpk    | Valve Pak                  | .vpk      | ✅         | ❌

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
