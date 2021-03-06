// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::{
    convert::{TryFrom, TryInto},
    ffi::{CStr, CString},
    marker::PhantomData,
    os::raw::{c_char, c_int},
};

use libcryptsetup_rs_sys::crypt_pbkdf_type;

use crate::{device::CryptDevice, err::LibcryptErr, Bool};

consts_to_from_enum!(
    /// Rust representation of random number generator enum
    CryptRngFlag,
    u32,
    Urandom => libcryptsetup_rs_sys::CRYPT_RNG_URANDOM,
    Random => libcryptsetup_rs_sys::CRYPT_RNG_RANDOM
);

/// Rust representation of key generator enum
pub enum CryptKdf {
    #[allow(missing_docs)]
    Pbkdf2,
    #[allow(missing_docs)]
    Argon2I,
    #[allow(missing_docs)]
    Argon2Id,
}

impl CryptKdf {
    /// Convert to a `char *` for C
    fn as_ptr(&self) -> *const c_char {
        match *self {
            CryptKdf::Pbkdf2 => libcryptsetup_rs_sys::CRYPT_KDF_PBKDF2.as_ptr() as *const c_char,
            CryptKdf::Argon2I => libcryptsetup_rs_sys::CRYPT_KDF_ARGON2I.as_ptr() as *const c_char,
            CryptKdf::Argon2Id => {
                libcryptsetup_rs_sys::CRYPT_KDF_ARGON2ID.as_ptr() as *const c_char
            }
        }
    }

    /// Convert from a C `char *`
    fn from_ptr(ptr: *const c_char) -> Result<Self, LibcryptErr> {
        if libcryptsetup_rs_sys::CRYPT_KDF_PBKDF2 == unsafe { CStr::from_ptr(ptr) }.to_bytes() {
            Ok(CryptKdf::Pbkdf2)
        } else if libcryptsetup_rs_sys::CRYPT_KDF_ARGON2I
            == unsafe { CStr::from_ptr(ptr) }.to_bytes()
        {
            Ok(CryptKdf::Argon2I)
        } else if libcryptsetup_rs_sys::CRYPT_KDF_ARGON2ID
            == unsafe { CStr::from_ptr(ptr) }.to_bytes()
        {
            Ok(CryptKdf::Argon2Id)
        } else {
            Err(LibcryptErr::InvalidConversion)
        }
    }
}

consts_to_from_enum!(
    /// Enum wrapping `CRYPT_PBKDF_*` flags
    CryptPbkdfFlag,
    u32,
    IterTimeSet => libcryptsetup_rs_sys::CRYPT_PBKDF_ITER_TIME_SET,
    NoBenchmark => libcryptsetup_rs_sys::CRYPT_PBKDF_NO_BENCHMARK
);

bitflags_to_from_struct!(
    /// Wrapper for a set of CryptPbkdfFlag
    CryptPbkdfFlags,
    CryptPbkdfFlag,
    u32
);

struct_ref_to_bitflags!(CryptPbkdfFlags, CryptPbkdfFlag, u32);

/// Rust representation of `crypt_pbkdf_type`
pub struct CryptPbkdfType {
    #[allow(missing_docs)]
    pub type_: CryptKdf,
    #[allow(missing_docs)]
    pub hash: String,
    #[allow(missing_docs)]
    pub time_ms: u32,
    #[allow(missing_docs)]
    pub iterations: u32,
    #[allow(missing_docs)]
    pub max_memory_kb: u32,
    #[allow(missing_docs)]
    pub parallel_threads: u32,
    #[allow(missing_docs)]
    pub flags: CryptPbkdfFlags,
}

impl TryFrom<libcryptsetup_rs_sys::crypt_pbkdf_type> for CryptPbkdfType {
    type Error = LibcryptErr;

    fn try_from(
        type_: libcryptsetup_rs_sys::crypt_pbkdf_type,
    ) -> Result<CryptPbkdfType, LibcryptErr> {
        Ok(CryptPbkdfType {
            type_: CryptKdf::from_ptr(type_.type_)?,
            hash: String::from(from_str_ptr!(type_.hash)?),
            time_ms: type_.time_ms,
            iterations: type_.iterations,
            max_memory_kb: type_.max_memory_kb,
            parallel_threads: type_.parallel_threads,
            flags: CryptPbkdfFlags::try_from(type_.flags)?,
        })
    }
}

impl<'a> TryFrom<&'a libcryptsetup_rs_sys::crypt_pbkdf_type> for CryptPbkdfType {
    type Error = LibcryptErr;

    fn try_from(v: &'a libcryptsetup_rs_sys::crypt_pbkdf_type) -> Result<Self, Self::Error> {
        Ok(CryptPbkdfType {
            type_: CryptKdf::from_ptr(v.type_)?,
            hash: from_str_ptr!(v.hash)?.to_string(),
            time_ms: v.time_ms,
            iterations: v.iterations,
            max_memory_kb: v.max_memory_kb,
            parallel_threads: v.parallel_threads,
            flags: CryptPbkdfFlags::try_from(v.flags)?,
        })
    }
}

/// A type wrapping a PBKDF type with pointers derived from Rust data types and lifetimes to ensure
/// pointer validity
pub struct CryptPbkdfTypeRef<'a> {
    /// Field containing a `crypt_pbkdf_type` that contains pointers valid for the supplied struct lifetime
    pub inner: crypt_pbkdf_type,
    phantomdata: PhantomData<&'a ()>,
}

impl<'a> CryptPbkdfTypeRef<'a> {
    /// Create a new `CryptPbkdfTypeRef` type
    pub fn new(inner: crypt_pbkdf_type) -> Self {
        CryptPbkdfTypeRef {
            inner,
            phantomdata: PhantomData,
        }
    }
}

impl<'a> TryInto<CryptPbkdfTypeRef<'a>> for &'a CryptPbkdfType {
    type Error = LibcryptErr;

    fn try_into(self) -> Result<CryptPbkdfTypeRef<'a>, Self::Error> {
        let inner = libcryptsetup_rs_sys::crypt_pbkdf_type {
            type_: self.type_.as_ptr(),
            hash: {
                let bytes = self.hash.as_bytes();
                CString::new(bytes)
                    .map_err(LibcryptErr::NullError)?
                    .as_ptr()
            },
            time_ms: self.time_ms,
            iterations: self.iterations,
            max_memory_kb: self.max_memory_kb,
            parallel_threads: self.parallel_threads,
            flags: (&self.flags).into(),
        };
        Ok(CryptPbkdfTypeRef {
            inner,
            phantomdata: PhantomData,
        })
    }
}

/// LUKS type (1 or 2)
pub enum LuksType {
    #[allow(missing_docs)]
    Luks1,
    #[allow(missing_docs)]
    Luks2,
}

impl LuksType {
    /// Convert Rust expression to an equivalent C pointer
    pub fn as_ptr(&self) -> *const c_char {
        match *self {
            LuksType::Luks1 => libcryptsetup_rs_sys::CRYPT_LUKS1.as_ptr() as *const c_char,
            LuksType::Luks2 => libcryptsetup_rs_sys::CRYPT_LUKS2.as_ptr() as *const c_char,
        }
    }
}

/// State of memory lock
pub enum LockState {
    #[allow(missing_docs)]
    Unlocked = 0,
    #[allow(missing_docs)]
    Locked = 1,
}

impl From<c_int> for LockState {
    fn from(v: c_int) -> Self {
        match v {
            i if i == LockState::Unlocked as c_int => LockState::Unlocked,
            _ => LockState::Locked,
        }
    }
}

/// Size allocated for metadata
#[derive(Debug, PartialEq, Eq)]
pub enum MetadataSize {
    #[allow(missing_docs)]
    Kb16 = 0x4_000,
    #[allow(missing_docs)]
    Kb32 = 0x8_000,
    #[allow(missing_docs)]
    Kb64 = 0x10_000,
    #[allow(missing_docs)]
    Kb128 = 0x20_000,
    #[allow(missing_docs)]
    Kb256 = 0x40_000,
    #[allow(missing_docs)]
    Kb512 = 0x80_000,
    #[allow(missing_docs)]
    Kb1024 = 0x100_000,
    #[allow(missing_docs)]
    Kb2048 = 0x200_000,
    #[allow(missing_docs)]
    Kb4096 = 0x400_000,
}

impl TryFrom<u64> for MetadataSize {
    type Error = LibcryptErr;

    fn try_from(v: u64) -> Result<Self, Self::Error> {
        let size = match v {
            i if i == MetadataSize::Kb16 as u64 => MetadataSize::Kb16,
            i if i == MetadataSize::Kb32 as u64 => MetadataSize::Kb32,
            i if i == MetadataSize::Kb64 as u64 => MetadataSize::Kb64,
            i if i == MetadataSize::Kb128 as u64 => MetadataSize::Kb128,
            i if i == MetadataSize::Kb256 as u64 => MetadataSize::Kb256,
            i if i == MetadataSize::Kb512 as u64 => MetadataSize::Kb512,
            i if i == MetadataSize::Kb1024 as u64 => MetadataSize::Kb1024,
            i if i == MetadataSize::Kb2048 as u64 => MetadataSize::Kb2048,
            i if i == MetadataSize::Kb4096 as u64 => MetadataSize::Kb4096,
            _ => return Err(LibcryptErr::InvalidConversion),
        };
        Ok(size)
    }
}

/// Size in KB for the allocated keyslot
pub struct KeyslotsSize(u64);

impl TryInto<u64> for KeyslotsSize {
    type Error = LibcryptErr;

    fn try_into(self) -> Result<u64, Self::Error> {
        if self.0 < 1 {
            return Err(LibcryptErr::InvalidConversion);
        }
        let converted = self.0 * (2 << 21);
        if converted > (2 << 26) {
            return Err(LibcryptErr::InvalidConversion);
        }
        Ok(converted)
    }
}

impl TryFrom<u64> for KeyslotsSize {
    type Error = LibcryptErr;

    fn try_from(v: u64) -> Result<Self, Self::Error> {
        let kbs = v / (2 << 21);
        if kbs > (2 << 26) || kbs < 1 {
            return Err(LibcryptErr::InvalidConversion);
        }
        Ok(KeyslotsSize(kbs))
    }
}

/// Handle to operate on cryptsetup device settings
pub struct CryptSettings<'a> {
    reference: &'a mut CryptDevice,
}

impl<'a> CryptSettings<'a> {
    pub(crate) fn new(reference: &'a mut CryptDevice) -> Self {
        CryptSettings { reference }
    }

    /// Set random number generator type
    pub fn set_rng_type(&mut self, rng_type: CryptRngFlag) {
        let rng_u32: u32 = rng_type.into();
        unsafe {
            libcryptsetup_rs_sys::crypt_set_rng_type(self.reference.as_ptr(), rng_u32 as c_int)
        }
    }

    /// Get random number generator type
    pub fn get_rng_type(&mut self) -> Result<CryptRngFlag, LibcryptErr> {
        CryptRngFlag::try_from(unsafe {
            libcryptsetup_rs_sys::crypt_get_rng_type(self.reference.as_ptr())
        } as u32)
    }

    /// Set PBKDF type
    pub fn set_pbkdf_type<'b>(
        &mut self,
        pbkdf_type: &'b CryptPbkdfType,
    ) -> Result<(), LibcryptErr> {
        let type_: CryptPbkdfTypeRef<'b> = pbkdf_type.try_into()?;
        errno!(unsafe {
            libcryptsetup_rs_sys::crypt_set_pbkdf_type(
                self.reference.as_ptr(),
                &type_.inner as *const crypt_pbkdf_type,
            )
        })
    }

    /// Get PBKDF parameters
    pub fn get_pbkdf_type_params(pbkdf_type: &CryptKdf) -> Result<CryptPbkdfType, LibcryptErr> {
        let type_ = ptr_to_result_with_reference!(unsafe {
            libcryptsetup_rs_sys::crypt_get_pbkdf_type_params(pbkdf_type.as_ptr())
        })?;
        CryptPbkdfType::try_from(type_)
    }

    /// Get PBKDF default type
    pub fn get_pbkdf_default(luks_type: &LuksType) -> Result<CryptPbkdfType, LibcryptErr> {
        let default = ptr_to_result_with_reference!(unsafe {
            libcryptsetup_rs_sys::crypt_get_pbkdf_default(luks_type.as_ptr())
        })?;
        CryptPbkdfType::try_from(default)
    }

    /// Get PBKDF type
    pub fn get_pbkdf_type(&mut self) -> Result<CryptPbkdfType, LibcryptErr> {
        let type_ = ptr_to_result_with_reference!(unsafe {
            libcryptsetup_rs_sys::crypt_get_pbkdf_type(self.reference.as_ptr())
        })?;
        CryptPbkdfType::try_from(type_)
    }

    /// Set the iteration time in milliseconds
    pub fn set_iteration_time(&mut self, iteration_time_ms: u64) {
        unsafe {
            libcryptsetup_rs_sys::crypt_set_iteration_time(
                self.reference.as_ptr(),
                iteration_time_ms,
            )
        }
    }

    /// Lock or unlock memory
    pub fn memory_lock(&mut self, lock: LockState) -> LockState {
        int_to_return!(
            unsafe {
                libcryptsetup_rs_sys::crypt_memory_lock(self.reference.as_ptr(), lock as c_int)
            },
            LockState
        )
    }

    /// Lock or unlock the metadata
    pub fn metadata_locking(&mut self, enable: Bool) -> Result<(), LibcryptErr> {
        errno!(unsafe {
            libcryptsetup_rs_sys::crypt_metadata_locking(self.reference.as_ptr(), enable as c_int)
        })
    }

    /// Set the metadata size and keyslot size
    pub fn set_metadata_size(
        &mut self,
        metadata_size: MetadataSize,
        keyslots_size: KeyslotsSize,
    ) -> Result<(), LibcryptErr> {
        errno!(unsafe {
            libcryptsetup_rs_sys::crypt_set_metadata_size(
                self.reference.as_ptr(),
                metadata_size as u64,
                keyslots_size.try_into()?,
            )
        })
    }

    /// Get the metadata size and keyslot size
    pub fn get_metadata_size(&mut self) -> Result<(MetadataSize, KeyslotsSize), LibcryptErr> {
        let mut metadata_size = 0u64;
        let mut keyslots_size = 0u64;
        errno!(unsafe {
            libcryptsetup_rs_sys::crypt_get_metadata_size(
                self.reference.as_ptr(),
                &mut metadata_size as *mut u64,
                &mut keyslots_size as *mut u64,
            )
        })?;
        let msize = MetadataSize::try_from(metadata_size)?;
        let ksize = KeyslotsSize::try_from(keyslots_size)?;
        Ok((msize, ksize))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_metadata_size() {
        assert_eq!(MetadataSize::try_from(0x4000).unwrap(), MetadataSize::Kb16);
        assert_eq!(MetadataSize::try_from(0x10000).unwrap(), MetadataSize::Kb64);
        assert!(MetadataSize::try_from(0x10001).is_err());
    }

    #[test]
    fn test_keyslots_size() {
        let size: u64 = KeyslotsSize(1).try_into().unwrap();
        assert_eq!(size, 2 << 21);
        let size: u64 = KeyslotsSize(5).try_into().unwrap();
        assert_eq!(size, 5 * (2 << 21));
        let ok: Result<u64, LibcryptErr> = KeyslotsSize(32).try_into();
        assert!(ok.is_ok());
        let not_ok: Result<u64, LibcryptErr> = KeyslotsSize(33).try_into();
        assert!(not_ok.is_err());
    }
}
