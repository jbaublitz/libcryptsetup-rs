use std::{
    convert::TryFrom,
    ffi::CStr,
    os::raw::{c_char, c_uint},
    path::PathBuf,
};

use crate::{device::CryptDevice, err::LibcryptErr};

use cryptsetup_sys::*;

/// Verity format flags
pub enum CryptVerity {
    NoHeader = cryptsetup_sys::CRYPT_VERITY_NO_HEADER as isize,
    CheckHash = cryptsetup_sys::CRYPT_VERITY_CHECK_HASH as isize,
    CreateHash = cryptsetup_sys::CRYPT_VERITY_CREATE_HASH as isize,
}

impl TryFrom<u32> for CryptVerity {
    type Error = LibcryptErr;

    fn try_from(v: u32) -> Result<Self, Self::Error> {
        Ok(match v {
            i if i == CryptVerity::NoHeader as u32 => CryptVerity::NoHeader,
            i if i == CryptVerity::CheckHash as u32 => CryptVerity::CheckHash,
            i if i == CryptVerity::CreateHash as u32 => CryptVerity::CreateHash,
            _ => return Err(LibcryptErr::InvalidConversion),
        })
    }
}

/// Wrapper for multiple `CRYPT_VERITY_*` flags
pub struct CryptVerityFlags(Vec<CryptVerity>);

bitflags_to_enum!(CryptVerityFlags, CryptVerity, u32);

/// Device formatting type options
pub enum Format {
    #[allow(missing_docs)]
    Plain,
    #[allow(missing_docs)]
    Luks1,
    #[allow(missing_docs)]
    Luks2,
    #[allow(missing_docs)]
    Loopaes,
    #[allow(missing_docs)]
    Verity,
    #[allow(missing_docs)]
    Tcrypt,
    #[allow(missing_docs)]
    Integrity,
}

pub struct CryptParamsVerity {
    pub hash_name: String,
    pub data_device: PathBuf,
    pub hash_device: PathBuf,
    pub fec_device: PathBuf,
    pub salt: Vec<u8>,
    pub hash_type: u32,
    pub data_block_size: u32,
    pub hash_block_size: u32,
    pub data_size: u64,
    pub hash_area_offset: u64,
    pub fec_area_offset: u64,
    pub fec_roots: u32,
    pub flags: CryptVerityFlags,
}

impl<'a> TryFrom<&'a cryptsetup_sys::crypt_params_verity> for CryptParamsVerity {
    type Error = LibcryptErr;

    fn try_from(v: &'a cryptsetup_sys::crypt_params_verity) -> Result<Self, Self::Error> {
        Ok(CryptParamsVerity {
            hash_name: from_str_ptr_to_owned!(v.hash_name)?,
            data_device: PathBuf::from(from_str_ptr_to_owned!(v.data_device)?),
            hash_device: PathBuf::from(from_str_ptr_to_owned!(v.hash_device)?),
            fec_device: PathBuf::from(from_str_ptr_to_owned!(v.fec_device)?),
            salt: Vec::from(unsafe {
                std::slice::from_raw_parts(v.salt as *const u8, v.salt_size as usize)
            }),
            hash_type: v.hash_type,
            data_block_size: v.data_block_size,
            hash_block_size: v.hash_block_size,
            data_size: v.data_size,
            hash_area_offset: v.hash_area_offset,
            fec_area_offset: v.fec_area_offset,
            fec_roots: v.fec_roots,
            flags: CryptVerityFlags::try_from(v.flags)?,
        })
    }
}

pub struct CryptParamsIntegrity {
    pub journal_size: u64,
    pub journal_watermark: c_uint,
    pub journal_commit_time: c_uint,
    pub interleave_sectors: u32,
    pub tag_size: u32,
    pub sector_size: u32,
    pub buffer_sectors: u32,
    pub integrity: String,
    pub integrity_key_size: u32,
    pub journal_integrity: String,
    pub journal_integrity_key: Vec<u8>,
    pub journal_crypt: String,
    pub journal_crypt_key: Vec<u8>,
}

impl<'a> TryFrom<&'a cryptsetup_sys::crypt_params_integrity> for CryptParamsIntegrity {
    type Error = LibcryptErr;

    fn try_from(v: &'a cryptsetup_sys::crypt_params_integrity) -> Result<Self, Self::Error> {
        Ok(CryptParamsIntegrity {
            journal_size: v.journal_size,
            journal_watermark: v.journal_watermark,
            journal_commit_time: v.journal_commit_time,
            interleave_sectors: v.interleave_sectors,
            tag_size: v.tag_size,
            sector_size: v.sector_size,
            buffer_sectors: v.buffer_sectors,
            integrity: from_str_ptr_to_owned!(v.integrity)?,
            integrity_key_size: v.integrity_key_size,
            journal_integrity: from_str_ptr_to_owned!(v.journal_integrity)?,
            journal_integrity_key: Vec::from(unsafe {
                std::slice::from_raw_parts(
                    v.journal_integrity_key as *const u8,
                    v.journal_integrity_key_size as usize,
                )
            }),
            journal_crypt: from_str_ptr_to_owned!(v.journal_crypt)?,
            journal_crypt_key: Vec::from(unsafe {
                std::slice::from_raw_parts(
                    v.journal_crypt_key as *const u8,
                    v.journal_crypt_key_size as usize,
                )
            }),
        })
    }
}

impl Format {
    /// Get `Format` as a char pointer
    pub(crate) fn as_ptr(&self) -> *const c_char {
        match *self {
            Format::Plain => cryptsetup_sys::CRYPT_PLAIN.as_ptr() as *const c_char,
            Format::Luks1 => cryptsetup_sys::CRYPT_LUKS1.as_ptr() as *const c_char,
            Format::Luks2 => cryptsetup_sys::CRYPT_LUKS2.as_ptr() as *const c_char,
            Format::Loopaes => cryptsetup_sys::CRYPT_LOOPAES.as_ptr() as *const c_char,
            Format::Verity => cryptsetup_sys::CRYPT_VERITY.as_ptr() as *const c_char,
            Format::Tcrypt => cryptsetup_sys::CRYPT_TCRYPT.as_ptr() as *const c_char,
            Format::Integrity => cryptsetup_sys::CRYPT_INTEGRITY.as_ptr() as *const c_char,
        }
    }

    /// Get `Format` from a char pointer
    fn from_ptr(p: *const c_char) -> Result<Self, LibcryptErr> {
        if cryptsetup_sys::CRYPT_PLAIN == unsafe { CStr::from_ptr(p) }.to_bytes() {
            Ok(Format::Plain)
        } else if cryptsetup_sys::CRYPT_LUKS1 == unsafe { CStr::from_ptr(p) }.to_bytes() {
            Ok(Format::Luks1)
        } else if cryptsetup_sys::CRYPT_LUKS2 == unsafe { CStr::from_ptr(p) }.to_bytes() {
            Ok(Format::Luks2)
        } else if cryptsetup_sys::CRYPT_LOOPAES == unsafe { CStr::from_ptr(p) }.to_bytes() {
            Ok(Format::Loopaes)
        } else if cryptsetup_sys::CRYPT_VERITY == unsafe { CStr::from_ptr(p) }.to_bytes() {
            Ok(Format::Verity)
        } else if cryptsetup_sys::CRYPT_TCRYPT == unsafe { CStr::from_ptr(p) }.to_bytes() {
            Ok(Format::Tcrypt)
        } else if cryptsetup_sys::CRYPT_INTEGRITY == unsafe { CStr::from_ptr(p) }.to_bytes() {
            Ok(Format::Integrity)
        } else {
            Err(LibcryptErr::InvalidConversion)
        }
    }
}

/// Handle for format operations on a device
pub struct CryptFormat<'a> {
    reference: &'a mut CryptDevice,
}

impl<'a> CryptFormat<'a> {
    pub(crate) fn new(reference: &'a mut CryptDevice) -> Self {
        CryptFormat { reference }
    }

    /// Get the formatting type
    pub fn get_type(&mut self) -> Result<Format, LibcryptErr> {
        Format::from_ptr(unsafe { crypt_get_type(self.reference.as_ptr()) })
    }

    /// Get the default formatting type
    pub fn get_default_type() -> Result<Format, LibcryptErr> {
        Format::from_ptr(unsafe { crypt_get_default_type() })
    }
}
