//! Open and read .dict or .dict.dz files.
//!
//! This module contains traits and structs to work with uncompressed .dict and compressed .dict.dz
//! files. These files contain the actual dictionary content. While these readers return the
//! definitions, they do not do any post-processing. Definitions are normally plain text, but they
//! could be HTML, or anything else, in theory (although plain text is the de facto default).
//!
//! To understand some of the constants defined in this module or to understand the internals of
//! the DictReaderDz struct, it is advisable to have a brief look at
//! [the GZip standard](https://tools.ietf.org/html/rfc1952).

use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use super::errors::DictError;
use byteorder::*;

/// Limit size of a word buffer, so that malicious index files cannot request too much memory for a
/// translation.
pub static MAX_BYTES_FOR_BUFFER: u64 = 1_048_576; // No headword definition is larger than 1M.

/// Byte mask to query for existence of FEXTRA field in the flags byte of a `.dz` file.
pub static GZ_FEXTRA: u8 = 0b0000_0100;
/// Byte mask to query for the existence of a file name in a `.dz` file.
pub static GZ_FNAME: u8 = 0b0000_1000; // Indicates whether a file name is contained in the archive.
/// Byte mask to query for the existence of a comment in a `.dz` file.
pub static GZ_COMMENT: u8 = 0b0001_0000; // Indicates, whether a comment is present.
/// Byte mask to detect that a comment is contained in a `.dz` file.
pub static GZ_FHCRC: u8 = 0b0000_0010;

/// A dictionary (content) reader.
///
/// This type abstracts from the underlying seek operations required for lookup
/// of headwords and provides easy methods to search for a word given a certain
/// offset and length. Users of a type which implements this trait don't need to care about compression
/// of the dictionary.
pub trait DictReader {
    /// Fetch the definition from the dictionary at offset and length.
    fn fetch_definition(&mut self, start_offset: u64, length: u64) -> Result<String, DictError>;
}

/// Raw Dict reader.
///
/// This reader can read uncompressed .dict files.
pub struct DictReaderRaw<B: Read + Seek> {
    dict_data: B,
    total_length: u64,
}

impl<B: Read + Seek> DictReaderRaw<B> {
    /// Get a new DictReader from a Reader.
    pub fn new(mut dict_data: B) -> Result<DictReaderRaw<B>, DictError> {
        let end = dict_data.seek(SeekFrom::End(0))?;
        Ok(DictReaderRaw {
            dict_data,
            total_length: end,
        })
    }
}

impl<B: Read + Seek> DictReader for DictReaderRaw<B> {
    /// Fetch definition from dictionary.
    fn fetch_definition(&mut self, start_offset: u64, length: u64) -> Result<String, DictError> {
        if length > MAX_BYTES_FOR_BUFFER {
            return Err(DictError::MemoryError);
        }

        if (start_offset + length) > self.total_length {
            return Err(DictError::IoError(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "a \
                      seek beyond the end of uncompressed data was requested",
            )));
        }

        self.dict_data.seek(SeekFrom::Start(start_offset))?;
        let mut read_data = vec![0; length as usize];
        let bytes_read = self.dict_data.read(read_data.as_mut_slice())? as u64;
        if bytes_read != length {
            // reading from end of file?
            return Err(DictError::IoError(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "seek beyond end of file",
            )));
        }
        Ok(String::from_utf8(read_data)?)
    }
}

/// Load a `DictReader` from file.
///
/// This function loads a `Dictreader` from a file and transparently selects
/// the correct reader using the file type extension, so the callee doesn't need to care about
/// compression (`.dz`).
///
/// # Errors
///
/// The function can return a `DictError`, which can either occur if a I/O error occurs, or when
/// the GZ compressed file is invalid.
pub fn load_dict<P: AsRef<Path>>(path: P) -> Result<Box<dyn DictReader>, DictError> {
    if path.as_ref().extension() == Some(OsStr::new("dz")) {
        let file = File::open(path.as_ref()).map_err(|e| {
            DictError::IoError(io::Error::other(format!(
                "can't open dictionary file {}: {}",
                path.as_ref().display(),
                e
            )))
        })?;
        Ok(Box::new(DictReaderDz::new(file)?))
    } else {
        let file = File::open(path.as_ref()).map_err(|e| {
            DictError::IoError(io::Error::other(format!(
                "can't open dictionary file {}: {}",
                path.as_ref().display(),
                e
            )))
        })?;
        let reader = BufReader::new(file);
        Ok(Box::new(DictReaderRaw::new(reader)?))
    }
}

/// Gzip Dict reader
///
/// This reader can read compressed .dict files with the file name suffix .dz.
/// This format is documented in RFC 1952 and in `man dictzip`. An example implementation can be
/// found in the dict daemon (dictd) in `data.c`.
pub struct DictReaderDz<B: Read + Seek> {
    /// Compressed DZ dictionary.
    dzdict: B,
    /// Length of an uncompressed chunk.
    uchunk_length: usize,
    /// End of compressed data.
    end_compressed_data: usize,
    /// Offsets in file where a new compressed chunk starts.
    chunk_offsets: Vec<usize>,
    /// Total size of uncompressed file.
    ufile_length: u64, // Has u64 to be quicker in comparing to offsets.
}

#[derive(Debug)]
// A (GZ) chunk, representing length and offset withing the compressed file.
struct Chunk {
    offset: usize,
    length: usize,
}

impl<B: Read + Seek> DictReaderDz<B> {
    /// Get a new DictReader from a Reader.
    pub fn new(dzdict: B) -> Result<DictReaderDz<B>, DictError> {
        let mut buffered_dzdict = BufReader::new(dzdict);
        let mut header = vec![0u8; 12];
        buffered_dzdict.read_exact(&mut header)?;
        Self::validate_gzip_header(&header)?;

        let flags = &header[3];
        let xlen = LittleEndian::read_u16(&header[10..12]);

        let mut fextra = vec![0u8; xlen as usize];
        buffered_dzdict.read_exact(&mut fextra)?;
        Self::validate_fextra_field(&fextra)?;

        let (uchunk_length, chunk_count) = Self::parse_fextra_header(&fextra)?;
        Self::validate_chunk_count(&fextra, chunk_count)?;

        Self::skip_optional_fields(&mut buffered_dzdict, flags)?;

        let (chunk_offsets, end_compressed_data) =
            Self::parse_chunk_offsets(&fextra, chunk_count, &mut buffered_dzdict)?;
        let uncompressed =
            Self::read_uncompressed_length(end_compressed_data, &mut buffered_dzdict)?;

        Ok(DictReaderDz {
            dzdict: buffered_dzdict.into_inner(),
            chunk_offsets,
            end_compressed_data,
            uchunk_length: uchunk_length as usize,
            ufile_length: uncompressed as u64,
        })
    }

    fn validate_gzip_header(header: &[u8]) -> Result<(), DictError> {
        if header[0..2] != [0x1F, 0x8B] {
            return Err(DictError::InvalidFileFormat(
                "Not in gzip format".into(),
                None,
            ));
        }

        let flags = &header[3];
        if (flags & GZ_FEXTRA) == 0 {
            return Err(DictError::InvalidFileFormat(
                "Extra flag (FLG.FEXTRA) not set, not in gzip + dzip format".into(),
                None,
            ));
        }
        Ok(())
    }

    fn validate_fextra_field(fextra: &[u8]) -> Result<(), DictError> {
        if fextra[0..2] != [b'R', b'A'] {
            return Err(DictError::InvalidFileFormat(
                "No dictzip info found in FEXTRA header".into(),
                None,
            ));
        }

        let xlen = fextra.len() as u16;
        let length_subfield = LittleEndian::read_u16(&fextra[2..4]);
        assert_eq!(
            length_subfield,
            xlen - 4,
            "the length of the subfield should be the same as the fextra field"
        );

        let subf_version = LittleEndian::read_u16(&fextra[4..6]);
        if subf_version != 1 {
            return Err(DictError::InvalidFileFormat(
                "Unimplemented dictzip version, only ver 1 supported".into(),
                None,
            ));
        }
        Ok(())
    }

    fn parse_fextra_header(fextra: &[u8]) -> Result<(u16, u16), DictError> {
        let uchunk_length = LittleEndian::read_u16(&fextra[6..8]);
        let chunk_count = LittleEndian::read_u16(&fextra[8..10]);
        if chunk_count == 0 {
            return Err(DictError::InvalidFileFormat(
                "No compressed chunks in file or broken header information".into(),
                None,
            ));
        }
        Ok((uchunk_length, chunk_count))
    }

    fn validate_chunk_count(fextra: &[u8], chunk_count: u16) -> Result<(), DictError> {
        let numbers_chunks_which_would_fit = ((fextra.len() - 10) / 2) as u16;
        if numbers_chunks_which_would_fit != chunk_count {
            return Err(DictError::InvalidFileFormat(
                format!(
                    "Expected {} chunks according to dictzip header, but the FEXTRA field can accomodate {}",
                    chunk_count, numbers_chunks_which_would_fit
                ),
                None,
            ));
        }
        Ok(())
    }

    fn skip_optional_fields(
        buffered_dzdict: &mut BufReader<B>,
        flags: &u8,
    ) -> Result<(), DictError> {
        if (flags & GZ_FNAME) != 0 {
            let mut tmp = Vec::new();
            buffered_dzdict.read_until(b'\0', &mut tmp)?;
        }

        if (flags & GZ_COMMENT) != 0 {
            let mut tmp = Vec::new();
            buffered_dzdict.read_until(b'\0', &mut tmp)?;
        }

        if (flags & GZ_FHCRC) != 0 {
            buffered_dzdict.seek(SeekFrom::Current(2))?;
        }
        Ok(())
    }

    fn parse_chunk_offsets(
        fextra: &[u8],
        chunk_count: u16,
        buffered_dzdict: &mut BufReader<B>,
    ) -> Result<(Vec<usize>, usize), DictError> {
        let mut chunk_offsets = Vec::with_capacity(chunk_count as usize);
        let mut end_compressed_data = buffered_dzdict.stream_position()? as usize;
        let chunks_from_header = &fextra[10usize..(10 + chunk_count * 2) as usize];

        for index in (0..chunks_from_header.len()).filter(|i| (i % 2) == 0) {
            let compressed_len =
                LittleEndian::read_u16(&chunks_from_header[index..(index + 2)]) as usize;
            chunk_offsets.push(end_compressed_data);
            end_compressed_data += compressed_len;
        }
        assert_eq!(
            chunk_offsets.len() as u16,
            chunk_count,
            "Chunk count mismatch"
        );

        Ok((chunk_offsets, end_compressed_data))
    }

    fn read_uncompressed_length(
        end_compressed_data: usize,
        buffered_dzdict: &mut BufReader<B>,
    ) -> Result<i32, DictError> {
        buffered_dzdict.seek(SeekFrom::Start(end_compressed_data as u64))?;
        Ok(buffered_dzdict.read_i32::<LittleEndian>()?)
    }

    fn get_chunks_for(&self, start_offset: u64, length: u64) -> Vec<Chunk> {
        let mut chunks = Vec::new();
        let start_chunk = start_offset as usize / self.uchunk_length;
        let end_chunk = (start_offset + length) as usize / self.uchunk_length;
        for id in start_chunk..=end_chunk {
            let chunk_length = match self.chunk_offsets.get(id + 1) {
                Some(next) => next - self.chunk_offsets[id],
                None => self.end_compressed_data - self.chunk_offsets[id],
            };
            chunks.push(Chunk {
                offset: self.chunk_offsets[id],
                length: chunk_length,
            });
        }

        chunks
    }

    // Inflate a dictdz chunk.
    fn inflate(&self, data: Vec<u8>) -> Result<Vec<u8>, DictError> {
        let mut decoder = flate2::Decompress::new(false);
        let mut decoded = vec![0u8; self.uchunk_length];
        decoder.decompress(
            data.as_slice(),
            decoded.as_mut_slice(),
            flate2::FlushDecompress::None,
        )?;
        Ok(decoded)
    }
}

impl<B: Read + Seek> DictReader for DictReaderDz<B> {
    // Fetch definition from the dictionary.
    fn fetch_definition(&mut self, start_offset: u64, length: u64) -> Result<String, DictError> {
        if length > MAX_BYTES_FOR_BUFFER {
            return Err(DictError::MemoryError);
        }
        if (start_offset + length) > self.ufile_length {
            return Err(DictError::IoError(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "a \
                      seek beyond the end of uncompressed data was requested",
            )));
        }
        let mut data = Vec::new();
        for chunk in self.get_chunks_for(start_offset, length) {
            let pos = self.dzdict.seek(SeekFrom::Start(chunk.offset as u64))?;
            if pos != (chunk.offset as u64) {
                return Err(DictError::IoError(io::Error::other(format!(
                    "attempted to seek to {} but new position is {}",
                    chunk.offset, pos
                ))));
            }
            let mut definition = vec![0u8; chunk.length];
            self.dzdict.read_exact(&mut definition)?;
            data.push(self.inflate(definition)?);
        }

        // Cut definition, convert to string.
        let cut_front = start_offset as usize % self.uchunk_length;
        // Join the chunks to one vector, only keeping the content of the definition.
        let data = match data.len() {
            0 => Vec::new(),
            1 => data[0][cut_front..cut_front + length as usize].to_vec(),
            n => {
                let mut tmp = data[0][cut_front..].to_vec();
                // First vec has been inserted into tmp, therefore skip first and last chunk, too.
                for text in data.iter().skip(1).take(n - 2) {
                    tmp.extend_from_slice(text);
                }
                // Add last chunk to tmp, omitting stuff after word definition end.
                let remaining_bytes = (length as usize + cut_front) % self.uchunk_length;
                tmp.extend_from_slice(&data[n - 1][..remaining_bytes]);
                tmp
            }
        };
        Ok(String::from_utf8(data)?)
    }
}
