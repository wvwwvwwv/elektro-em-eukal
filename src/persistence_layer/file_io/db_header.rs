// SPDX-FileCopyrightText: 2023 Changgyoo Park <wvwwvwwv@me.com>
//
// SPDX-License-Identifier: Apache-2.0

//! The header of the database file.

use super::RandomAccessFile;
use crate::Error;
use std::sync::atomic::Ordering::Relaxed;

/// The header of the database file.
#[derive(Debug)]
pub struct DatabaseHeader {
    /// Version.
    pub version: u64,

    /// The offset in the file that indicates the start page of the log space.
    pub log_offset: u64,

    /// The offset in the file that indicates the start page of the directory space.
    pub directory_offset: u64,

    /// A linked list of free pages.
    pub free_page_link: u64,
}

/// The database version.
pub const VERSION: u64 = 1;

/// The size of a page.
pub const PAGE_SIZE: u64 = 1_u64 << 9;

impl DatabaseHeader {
    /// Reads the header from the database file.
    ///
    /// It writes the header information into the file if none present.
    #[inline]
    pub fn from_file(db: &RandomAccessFile) -> Result<Self, Error> {
        let mut buffer = [0_u8; 8];
        if db.len(Relaxed) == 0 {
            let db_header = DatabaseHeader::default();
            db_header.flush_header(db)?;
            Ok(db_header)
        } else {
            db.read(&mut buffer, 0).map_err(|e| Error::IO(e.kind()))?;
            let version = u64::from_le_bytes(buffer);
            db.read(&mut buffer, 8).map_err(|e| Error::IO(e.kind()))?;
            let log_offset = u64::from_le_bytes(buffer);
            db.read(&mut buffer, 16).map_err(|e| Error::IO(e.kind()))?;
            let directory_offset = u64::from_le_bytes(buffer);
            db.read(&mut buffer, 24).map_err(|e| Error::IO(e.kind()))?;
            let free_page_link = u64::from_le_bytes(buffer);
            Ok(Self {
                version,
                log_offset,
                directory_offset,
                free_page_link,
            })
        }
    }

    /// Flushes the content of the [`DatabaseHeader`] to the database file.
    #[allow(dead_code)]
    #[inline]
    pub fn flush_header(&self, db: &RandomAccessFile) -> Result<(), Error> {
        if db.len(Relaxed) < PAGE_SIZE * 3 {
            db.set_len(PAGE_SIZE * 3).map_err(|e| Error::IO(e.kind()))?;
        }
        let mut buffer;
        buffer = self.version.to_le_bytes();
        db.write(&buffer, 0).map_err(|e| Error::IO(e.kind()))?;
        buffer = self.log_offset.to_le_bytes();
        db.write(&buffer, 8).map_err(|e| Error::IO(e.kind()))?;
        buffer = self.directory_offset.to_le_bytes();
        db.write(&buffer, 16).map_err(|e| Error::IO(e.kind()))?;
        buffer = self.free_page_link.to_le_bytes();
        db.write(&buffer, 24).map_err(|e| Error::IO(e.kind()))?;
        db.sync_all().map_err(|e| Error::IO(e.kind()))?;
        Ok(())
    }
}

impl Default for DatabaseHeader {
    #[inline]
    fn default() -> Self {
        Self {
            version: VERSION,
            log_offset: PAGE_SIZE,
            directory_offset: PAGE_SIZE * 2,
            free_page_link: Default::default(),
        }
    }
}
