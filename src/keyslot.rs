// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::{
    convert::TryFrom,
    os::raw::c_int,
    path::{Path, PathBuf},
    ptr,
};

use crate::{
    device::CryptDevice, err::LibcryptErr, format::EncryptionFormat, settings::CryptPbkdfType,
};

consts_to_from_enum!(
    /// Flags for tunable options when operating with volume keys
    CryptVolumeKeyFlag,
    u32,
    NoSegment => libcryptsetup_rs_sys::CRYPT_VOLUME_KEY_NO_SEGMENT,
    Set => libcryptsetup_rs_sys::CRYPT_VOLUME_KEY_SET,
    DigestReuse => libcryptsetup_rs_sys::CRYPT_VOLUME_KEY_DIGEST_REUSE
);

bitflags_to_from_struct!(
    /// Set of volume key flags
    CryptVolumeKeyFlags,
    CryptVolumeKeyFlag,
    u32
);

consts_to_from_enum!(
    /// Value indicating the status of a keyslot
    KeyslotInfo,
    u32,
    Invalid => libcryptsetup_rs_sys::crypt_keyslot_info_CRYPT_SLOT_INVALID,
    Inactive => libcryptsetup_rs_sys::crypt_keyslot_info_CRYPT_SLOT_INACTIVE,
    Active => libcryptsetup_rs_sys::crypt_keyslot_info_CRYPT_SLOT_ACTIVE,
    ActiveLast => libcryptsetup_rs_sys::crypt_keyslot_info_CRYPT_SLOT_ACTIVE_LAST,
    Unbound => libcryptsetup_rs_sys::crypt_keyslot_info_CRYPT_SLOT_UNBOUND
);

consts_to_from_enum!(
    /// Value indicating the priority of a keyslot
    KeyslotPriority,
    i32,
    Invalid => libcryptsetup_rs_sys::crypt_keyslot_priority_CRYPT_SLOT_PRIORITY_INVALID,
    Ignore => libcryptsetup_rs_sys::crypt_keyslot_priority_CRYPT_SLOT_PRIORITY_IGNORE,
    Normal => libcryptsetup_rs_sys::crypt_keyslot_priority_CRYPT_SLOT_PRIORITY_NORMAL,
    Prefer => libcryptsetup_rs_sys::crypt_keyslot_priority_CRYPT_SLOT_PRIORITY_PREFER
);

/// Handle for keyslot operations
pub struct CryptKeyslot<'a> {
    reference: &'a mut CryptDevice,
    keyslot: c_int,
}

impl<'a> CryptKeyslot<'a> {
    pub(crate) fn new(reference: &'a mut CryptDevice, keyslot: Option<c_int>) -> Self {
        CryptKeyslot {
            reference,
            keyslot: keyslot.unwrap_or(libcryptsetup_rs_sys::CRYPT_ANY_SLOT),
        }
    }

    /// Add key slot using a passphrase
    pub fn add_by_passphrase(
        &mut self,
        passphrase: &[u8],
        new_passphrase: &[u8],
    ) -> Result<c_int, LibcryptErr> {
        errno_int_success!(unsafe {
            libcryptsetup_rs_sys::crypt_keyslot_add_by_passphrase(
                self.reference.as_ptr(),
                self.keyslot,
                to_byte_ptr!(passphrase),
                passphrase.len(),
                to_byte_ptr!(new_passphrase),
                new_passphrase.len(),
            )
        })
    }

    /// Change allocated key slot using a passphrase
    pub fn change_by_passphrase(
        &mut self,
        keyslot_old: c_int,
        keyslot_new: c_int,
        passphrase: &[u8],
        new_passphrase: &[u8],
    ) -> Result<c_int, LibcryptErr> {
        errno_int_success!(unsafe {
            libcryptsetup_rs_sys::crypt_keyslot_change_by_passphrase(
                self.reference.as_ptr(),
                keyslot_old,
                keyslot_new,
                to_byte_ptr!(passphrase),
                passphrase.len(),
                to_byte_ptr!(new_passphrase),
                new_passphrase.len(),
            )
        })
    }

    /// Add key slot using key file
    pub fn add_by_keyfile_device_offset(
        &mut self,
        keyfile_and_size: (&Path, crate::size_t),
        keyfile_offset: u64,
        new_keyfile_and_size: (&Path, crate::size_t),
        new_keyfile_offset: u64,
    ) -> Result<c_int, LibcryptErr> {
        let (keyfile, keyfile_size) = keyfile_and_size;
        let (new_keyfile, new_keyfile_size) = new_keyfile_and_size;
        let keyfile_cstring = path_to_cstring!(keyfile)?;
        let new_keyfile_cstring = path_to_cstring!(new_keyfile)?;
        errno_int_success!(unsafe {
            libcryptsetup_rs_sys::crypt_keyslot_add_by_keyfile_device_offset(
                self.reference.as_ptr(),
                self.keyslot,
                keyfile_cstring.as_ptr(),
                keyfile_size,
                keyfile_offset,
                new_keyfile_cstring.as_ptr(),
                new_keyfile_size,
                new_keyfile_offset,
            )
        })
    }

    /// Add key slot with a key
    pub fn add_by_key(
        &mut self,
        volume_key: Option<&[u8]>,
        passphrase: &[u8],
        flags: CryptVolumeKeyFlags,
    ) -> Result<c_int, LibcryptErr> {
        let (vk_ptr, vk_len) = match volume_key {
            Some(vk) => (to_byte_ptr!(vk), vk.len()),
            None => (std::ptr::null(), 0),
        };
        errno_int_success!(unsafe {
            libcryptsetup_rs_sys::crypt_keyslot_add_by_key(
                self.reference.as_ptr(),
                self.keyslot,
                vk_ptr,
                vk_len,
                to_byte_ptr!(passphrase),
                passphrase.len(),
                flags.into(),
            )
        })
    }

    /// Destroy key slot
    pub fn destroy(&mut self) -> Result<(), LibcryptErr> {
        errno!(unsafe {
            libcryptsetup_rs_sys::crypt_keyslot_destroy(self.reference.as_ptr(), self.keyslot)
        })
    }

    /// Get keyslot status
    pub fn status(&mut self) -> Result<KeyslotInfo, LibcryptErr> {
        try_int_to_return!(
            unsafe {
                libcryptsetup_rs_sys::crypt_keyslot_status(self.reference.as_ptr(), self.keyslot)
            },
            KeyslotInfo
        )
    }

    /// Get keyslot priority (LUKS2 specific)
    pub fn get_priority(&mut self) -> Result<KeyslotPriority, LibcryptErr> {
        try_int_to_return!(
            unsafe {
                libcryptsetup_rs_sys::crypt_keyslot_get_priority(
                    self.reference.as_ptr(),
                    self.keyslot,
                )
            },
            KeyslotPriority
        )
    }

    /// Get keyslot priority (LUKS2 specific)
    pub fn set_priority(&mut self, priority: KeyslotPriority) -> Result<(), LibcryptErr> {
        errno!(unsafe {
            libcryptsetup_rs_sys::crypt_keyslot_set_priority(
                self.reference.as_ptr(),
                self.keyslot,
                priority as i32,
            )
        })
    }

    /// Get maximum keyslots supported for device type
    pub fn max_keyslots(fmt: EncryptionFormat) -> Result<c_int, LibcryptErr> {
        errno_int_success!(unsafe { libcryptsetup_rs_sys::crypt_keyslot_max(fmt.as_ptr()) })
    }

    /// Get keyslot area pointers
    pub fn area(&mut self) -> Result<(u64, u64), LibcryptErr> {
        let mut offset = 0u64;
        let mut length = 0u64;
        errno!(unsafe {
            libcryptsetup_rs_sys::crypt_keyslot_area(
                self.reference.as_ptr(),
                self.keyslot,
                &mut offset as *mut u64,
                &mut length as *mut u64,
            )
        })
        .map(|_| (offset, length))
    }

    /// Get size of key in keyslot - only different from `crypt_get_volume_key_size()` binding
    /// in the case of LUKS2 using unbound keyslots
    pub fn get_key_size(&mut self) -> Result<c_int, LibcryptErr> {
        errno_int_success!(unsafe {
            libcryptsetup_rs_sys::crypt_keyslot_get_key_size(self.reference.as_ptr(), self.keyslot)
        })
    }

    /// Get encryption cipher and key size of keyslot (not data)
    pub fn get_encryption(&mut self) -> Result<(&str, crate::size_t), LibcryptErr> {
        let mut key_size: crate::size_t = 0;
        ptr_to_result!(unsafe {
            libcryptsetup_rs_sys::crypt_keyslot_get_encryption(
                self.reference.as_ptr(),
                self.keyslot,
                &mut key_size as *mut crate::size_t,
            )
        })
        .and_then(|ptr| from_str_ptr!(ptr))
        .map(|st| (st, key_size))
    }

    /// Get PBDKF parameters for a keyslot
    pub fn get_pbkdf(&mut self) -> Result<CryptPbkdfType, LibcryptErr> {
        let mut type_ = libcryptsetup_rs_sys::crypt_pbkdf_type {
            type_: ptr::null(),
            hash: ptr::null(),
            time_ms: 0,
            iterations: 0,
            max_memory_kb: 0,
            parallel_threads: 0,
            flags: 0,
        };
        errno!(unsafe {
            libcryptsetup_rs_sys::crypt_keyslot_get_pbkdf(
                self.reference.as_ptr(),
                self.keyslot,
                &mut type_ as *mut _,
            )
        })
        .and_then(|_| CryptPbkdfType::try_from(type_))
    }

    /// Set encryption used for keyslot
    pub fn set_encryption(
        &mut self,
        cipher: &str,
        key_size: crate::size_t,
    ) -> Result<(), LibcryptErr> {
        let cipher_cstring = to_cstring!(cipher)?;
        errno!(unsafe {
            libcryptsetup_rs_sys::crypt_keyslot_set_encryption(
                self.reference.as_ptr(),
                cipher_cstring.as_ptr(),
                key_size,
            )
        })
    }

    /// Get directory where crypt devices are mapped
    pub fn get_dir() -> Result<Box<Path>, LibcryptErr> {
        ptr_to_result!(unsafe { libcryptsetup_rs_sys::crypt_get_dir() })
            .and_then(|s| from_str_ptr_to_owned!(s))
            .map(PathBuf::from)
            .map(|b| b.into_boxed_path())
    }
}
