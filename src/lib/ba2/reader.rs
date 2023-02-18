use std::{
    fs,
    io::{BufReader, Read, Seek, SeekFrom},
    path::Path,
    str,
};

use libflate::zlib;

use crate::{FileType, ReadEx};

use super::records::{
    GeneralBlock, Header, TextureBlock, TextureChunk, TextureInfo, BA2_SIGNATURE,
};

pub struct Reader {
    stm: BufReader<fs::File>,
    general_files: Vec<GeneralBlock>,
    texture_files: Vec<TextureInfo>,
    names: Vec<String>,
}

impl Reader {
    fn open(path: &Path, options: crate::reader::Options) -> crate::reader::Result<Self> {
        let file = fs::File::open(path).map_err(crate::reader::Error::OpeningInputFile)?;
        let mut stm = BufReader::new(file);

        let hdr = Header::read(&mut stm).map_err(crate::reader::Error::ReadingHeader)?;

        if hdr.signature != BA2_SIGNATURE.as_bytes() {
            return Err(crate::reader::Error::InvalidStringSignature {
                signature: String::from_utf8_lossy(&hdr.signature).to_string(),
                expected_signature: BA2_SIGNATURE,
            });
        }

        if hdr.version != 1 {
            return Err(crate::reader::Error::UnsupportedVersion {
                version: hdr.version,
                supported_versions: &[1u32],
            });
        }

        let archive_type = str::from_utf8(&hdr.archive_type)
            .map_err(|_| crate::reader::Error::Other("invalid block type".into()))?;

        let mut general_files = Vec::new();
        let mut texture_files = Vec::new();

        match archive_type {
            "GNRL" => {
                for _ in 0..hdr.num_files {
                    let rec = GeneralBlock::read(&mut stm)
                        .map_err(crate::reader::Error::ReadingInputFile)?;
                    general_files.push(rec);
                }
            }
            "DX10" => {
                for _ in 0..hdr.num_files {
                    let texture = TextureBlock::read(&mut stm)
                        .map_err(crate::reader::Error::ReadingInputFile)?;
                    let mut chunks = Vec::new();

                    for _ in 0..texture.num_chunks {
                        let texture_chunk = TextureChunk::read(&mut stm)
                            .map_err(crate::reader::Error::ReadingInputFile)?;
                        chunks.push(texture_chunk);
                    }

                    texture_files.push(TextureInfo {
                        hdr: texture,
                        chunks,
                    });
                }

                return Err(crate::reader::Error::Unsupported(
                    "texture archives are not supported",
                ));
            }
            "GNMF" => {
                return Err(crate::reader::Error::Unsupported(
                    "GNMF archives are not supported",
                ));
            }
            _ => {
                return Err(crate::reader::Error::InvalidHeader(format!(
                    "unknown archive type '{}'",
                    archive_type
                )));
            }
        }

        stm.seek(SeekFrom::Start(hdr.names_offset))
            .map_err(crate::reader::Error::ReadingInputFile)?;

        let mut names = Vec::new();

        for _ in 0..hdr.num_files {
            let name = stm
                .read_u16le_string()
                .map_err(crate::reader::Error::ReadingFileName)?;
            names.push(name);
        }

        if options.strict {
            for (_index, file) in general_files.iter().enumerate() {
                if file.padding != 0xBAADF00D {
                    return Err(crate::reader::Error::Other("invalid padding".into()));
                }

                // let name = &names[index];
                // let (dir, fname) = name.rsplit_once('/').unwrap_or_else(|| ("", name.as_str()));
                // let (fname, _ext) = fname.rsplit_once('.').unwrap_or_else(|| (fname, ""));
                // let dir_crc = crc32fast::hash(dir.to_lowercase().as_bytes());
                // let fname_crc = crc32fast::hash(fname.to_lowercase().as_bytes());
            }
        }

        Ok(Reader {
            stm,
            general_files,
            texture_files,
            names,
        })
    }
}

impl crate::reader::Reader for Reader {
    fn len(&self) -> usize {
        let len = self.general_files.len();

        if len > 0 {
            return len;
        }

        self.texture_files.len()
    }

    fn get_file(&self, index: usize) -> crate::reader::File {
        if self.general_files.len() > 0 {
            let file = self
                .general_files
                .get(index)
                .expect("`index` should be within boundaries");

            crate::reader::File {
                name: self.names[index].clone(),
                file_type: FileType::RegularFile,
                size: Some(file.unpacked_size.into()),
            }
        } else {
            let file = self
                .texture_files
                .get(index)
                .expect("`index` should be within boundaries");

            crate::reader::File {
                name: self.names[index].clone(),
                file_type: FileType::RegularFile,
                size: Some(file.chunks[0].unpacked_size.into()),
            }
        }
    }

    fn open_file_by_index<'a>(
        &'a mut self,
        index: usize,
    ) -> crate::reader::Result<Box<dyn Read + 'a>> {
        if self.general_files.len() > 0 {
            let file = self
                .general_files
                .get(index)
                .expect("`index` should be within boundaries");

            self.stm
                .seek(SeekFrom::Start(file.offset))
                .map_err(crate::reader::Error::ReadingInputFile)?;

            let stm = self.stm.by_ref().take(file.packed_size.into());
            let rdr = zlib::Decoder::new(stm).map_err(crate::reader::Error::ReadingInputFile)?;

            return Ok(Box::new(rdr));
        } else {
            return Err(crate::reader::Error::Unsupported(
                "texture archives are not supported",
            ));
            /*
                        let file = self
                            .texture_files
                            .get(index)
                            .expect("`index` should be within boundaries");
                        let chuck = &file.chunks[0];

                        self.stm.seek(SeekFrom::Start(chuck.offset))
                            .map_err(crate::reader::Error::ReadingInputFile)?;

                        let stm = self.stm.by_ref().take(chuck.packed_size.into());
                        let rdr = zlib::Decoder::new(stm)
                            .map_err(crate::reader::Error::ReadingInputFile)?;
                        return Ok(Box::new(rdr))
            */
        }
    }
}

pub fn make_reader(
    path: &Path,
    options: crate::reader::Options,
) -> crate::reader::Result<Box<dyn crate::reader::Reader>> {
    Ok(Box::new(Reader::open(path, options)?))
}
