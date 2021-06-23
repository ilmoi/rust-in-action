use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::collections::HashMap;
use std::path::Path;
use std::io;
use std::io::{BufReader, Seek, SeekFrom, Read, BufWriter, Write};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use crc::{crc32};

type ByteString = Vec<u8>; //like String but not guaranteed to be utf-8
type ByteStr = [u8]; //like &str but not guaranteed to be utf-8

// for an example of where invalid utf-8 causes String to error see this -> https://people.gnome.org/~federico/blog/correctness-in-rust-reading-strings.html

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyValuePair {
    pub key: ByteString,
    pub value: ByteString,
}

#[derive(Debug)]
pub struct ActionKV {
    f: File,
    pub index: HashMap<ByteString, u64>,
}

impl ActionKV {
    pub fn open(path: &Path) -> io::Result<Self> {
        // opens the file in append only mode
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .append(true)
            .open(path)?;
        // creates an index in the form of a hashmap
        let index = HashMap::new();
        Ok(Self{ f, index })
    }
    /// populates the index with key-value pairs and where they sit in the file
    /// what if a kv appears twice: earlier and later in the file?
    /// later read will take precedence, which is the whole idea behind an append-only log-based data store
    pub fn load(&mut self) -> io::Result<()> {
        //performs large, infrequent reads on the underlying Read and maintains an in-memory buffer of the results.
        let mut f = BufReader::new(&mut self.f);

        loop {
            //The Seek trait provides a cursor which can be moved within a stream of bytes.
            //SeekFrom::Current Sets the offset to the current position plus the specified number of bytes.
            let current_position = f.seek(SeekFrom::Current(0))?;

            //try to process the next key value from the current position
            //to actually process the record we're using an implementation of the Bitcask storage standard
            //it's nosql, slow, but guarantees it will never lose / compromise data
            let maybe_kv = ActionKV::process_record(&mut f);
            let kv = match maybe_kv {
                Ok(kv) => kv,
                Err(e) => {
                    match e.kind() {
                        // if reach EOF we break out of the loop
                        io::ErrorKind::UnexpectedEof => break,
                        // for all other errors return the error itself
                        _ => return Err(e),
                    }
                },
            };

            //if kv processed successfully, insert it into the index so it can be quickly found later
            self.index.insert(kv.key, current_position);
        }

        Ok(())
    }

    /// takes anything that implements the Read trait - could be a file, but could also be a [u8]
    fn process_record<R: Read>(f: &mut R) -> io::Result<KeyValuePair> {
        // remember we're passing in a stream of bytes
        // but it's important which way bytes are formatted - Little or Big endian
        // here we ensure they're read as LittleEndian, plucking the first 3x 4 bytes = 12 bytes (header in Bitcask)
        let saved_checksum = f.read_u32::<LittleEndian>()?;
        let key_len = f.read_u32::<LittleEndian>()?;
        let val_len = f.read_u32::<LittleEndian>()?;
        let data_len = key_len + val_len;

        // allocated enough space to store our data
        let mut data = ByteString::with_capacity(data_len as usize);

        // new scope so that we can drop() the reference to f after we're done
        // the point of this is that after we've dropped, the original f is still intact and can be read again
        // why do we care to do this? because this fn is called in a loop in load() and we can't consume f the first time we do it
        {
            // take a &mut ref to f
            f.by_ref()
                // decide to read data_len worth of data
                .take(data_len as u64)
                // do it - read into data, which we defined earlier
                .read_to_end(&mut data)?;
        }

        debug_assert_eq!(data.len(), data_len as usize);

        //we're using a particular kind of checksum here, crc32. More complex than parit bit, but less complex than crypto hash fns
        //this part is what gives Bitcask it's resiliency and no corruption guarantees
        let checksum = crc32::checksum_ieee(&data);
        if checksum != saved_checksum {
            panic!("checksums don't match");
        }

        // split vector into K and V
        let value = data.split_off(key_len as usize);
        let key = data;

        Ok(KeyValuePair {key, value})
    }

    pub fn get(&mut self, key: &ByteStr) -> io::Result<Option<ByteString>> {
        let position = match self.index.get(key) {
            None => return Ok(None),
            Some(position) => *position,
        };

        let kv = self.get_at(position)?;

        Ok(Some(kv.value))

    }

    pub fn get_at(&mut self, position: u64) -> io::Result<KeyValuePair> {
        // move f into mem
        let mut f = BufReader::new(&mut self.f);
        // go to position
        f.seek(SeekFrom::Start(position))?;
        // process and return the record
        let kv = ActionKV::process_record(&mut f)?;
        Ok(kv)
    }

    pub fn insert(&mut self, key: &ByteStr, value: &ByteStr) -> io::Result<()> {
        //insert the actual record
        let position = self.insert_but_ignore_index(key, value)?;
        //update the index
        self.index.insert(key.to_vec(), position);
        Ok(())
    }

    fn insert_but_ignore_index(&mut self, key: &ByteStr, value: &ByteStr) -> io::Result<u64> {
        // move f into mem
        let mut f = BufWriter::new(&mut self.f);

        // create a tmp buffer with enough space
        let key_len = key.len();
        let val_len = value.len();
        let data_len = key_len + val_len;
        let mut tmp = ByteString::with_capacity(data_len);

        //write key and value contiguously into that buffer
        for byte in key {
            tmp.push(*byte);
        }
        for byte in value {
            tmp.push(*byte);
        }

        // prep the checksum
        let checksum = crc32::checksum_ieee(&tmp);

        //move to the end of the file
        let next_byte = SeekFrom::End(0);
        //now seek from the current position (which is the end of the file, since we just moved there)
        let current_position = f.seek(SeekFrom::Current(0))?;
        //move to next byte
        f.seek(next_byte)?;

        //write header (12 bytes: checksum, key len, val len)
        f.write_u32::<LittleEndian>(checksum)?;
        f.write_u32::<LittleEndian>(key_len as u32)?;
        f.write_u32::<LittleEndian>(val_len as u32)?;
        //write body
        f.write_all(&mut tmp)?;

        Ok(current_position)
    }

    pub fn update(&mut self, key: &ByteStr, value: &ByteStr) -> io::Result<()> {
        //we simply insert the new value since this is an append only data store
        self.insert(key, value)
    }

    pub fn delete(&mut self, key: &ByteStr) -> io::Result<()> {
        //we simply insert an EMPTY value since this is an append only data store
        self.insert(key, b"")
    }
}