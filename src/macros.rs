// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/// Convert an errno-zero-success return pattern into a `Result<(), LibcryptErr>`
macro_rules! errno {
    ( $rc:expr ) => {
        match $rc {
            i if i < 0 => {
                return Err($crate::err::LibcryptErr::IOError(
                    std::io::Error::from_raw_os_error(-i),
                ))
            }
            i if i > 0 => panic!("Unexpected return value {}", i),
            _ => Result::<(), $crate::err::LibcryptErr>::Ok(()),
        }
    };
}

/// Convert an errno-positive-int-success return pattern into a `Result<std::os::raw::c_int, LibcryptErr>`
macro_rules! errno_int_success {
    ( $rc:expr ) => {
        match $rc {
            i if i < 0 => {
                return Err($crate::err::LibcryptErr::IOError(
                    std::io::Error::from_raw_os_error(-i),
                ))
            }
            i => Result::<_, $crate::err::LibcryptErr>::Ok(i),
        }
    };
}

/// Convert an integer return value into specified type
macro_rules! int_to_return {
    ( $rc:expr, $type:ty ) => {
        <$type>::from($rc)
    };
}

/// Try converting an integer return value into specified type
macro_rules! try_int_to_return {
    ( $rc:expr, $type:ty ) => {
        <$type>::try_from($rc)
    };
}

/// Convert a pointer to an `Option` containing a pointer
macro_rules! ptr_to_option {
    ( $ptr:expr ) => {{
        let p = $ptr;
        if p.is_null() {
            None
        } else {
            Some(p)
        }
    }};
}

/// Convert a pointer to an `Result` containing a pointer
macro_rules! ptr_to_result {
    ( $ptr:expr ) => {{
        ptr_to_option!($ptr).ok_or($crate::err::LibcryptErr::NullPtr)
    }};
}

/// Convert a pointer to a `Result` containing a reference
macro_rules! ptr_to_result_with_reference {
    ( $ptr:expr ) => {{
        let p = $ptr;
        unsafe { p.as_ref() }.ok_or($crate::err::LibcryptErr::NullPtr)
    }};
}

/// Convert a `Path` type into `CString`
macro_rules! path_to_cstring {
    ( $path:expr ) => {
        match $path
            .to_str()
            .ok_or_else(|| LibcryptErr::InvalidConversion)
            .and_then(|s| std::ffi::CString::new(s).map_err(LibcryptErr::NullError))
        {
            Ok(s) => Ok(s),
            Err(e) => Err(e),
        }
    };
}

/// Convert a string type into `CString`
macro_rules! to_cstring {
    ( $str:expr ) => {
        match std::ffi::CString::new($str.as_bytes()) {
            Ok(s) => Ok(s),
            Err(e) => Err($crate::err::LibcryptErr::NullError(e)),
        }
    };
}

/// Convert a byte slice into `*const c_char`
macro_rules! to_byte_ptr {
    ( $bytes:expr ) => {
        $bytes.as_ptr() as *const std::os::raw::c_char
    };
}

/// Convert a byte slice into `*mut c_char`
macro_rules! to_mut_byte_ptr {
    ( $bytes:expr ) => {
        $bytes.as_mut_ptr() as *mut std::os::raw::c_char
    };
}

/// Convert a `*const c_char` into a `&str` type
macro_rules! from_str_ptr {
    ( $str_ptr:expr ) => {
        unsafe { ::std::ffi::CStr::from_ptr($str_ptr) }
            .to_str()
            .map_err($crate::err::LibcryptErr::Utf8Error)
    };
}

/// Convert a `*const c_char` into a `String` type
macro_rules! from_str_ptr_to_owned {
    ( $str_ptr:expr ) => {
        unsafe { ::std::ffi::CStr::from_ptr($str_ptr) }
            .to_str()
            .map_err($crate::err::LibcryptErr::Utf8Error)
            .map(|s| s.to_string())
    };
}

/// Convert constants to and from a flag enum
macro_rules! consts_to_from_enum {
    ( #[$meta:meta] $flag_enum:ident, $flag_type:ty, $( $name:ident => $constant:expr ),* ) => {
        #[$meta]
        #[derive(Copy, Clone)]
        pub enum $flag_enum {
            $(
                #[allow(missing_docs)]
                $name,
            )*
        }

        impl std::convert::Into<$flag_type> for $flag_enum {
            fn into(self) -> $flag_type {
                match self {
                    $(
                        $flag_enum::$name => $constant,
                    )*
                }
            }
        }

        impl std::convert::TryFrom<$flag_type> for $flag_enum {
            type Error = $crate::err::LibcryptErr;

            fn try_from(v: $flag_type) -> Result<Self, Self::Error> {
                Ok(match v {
                    $(
                        i if i == $constant => $flag_enum::$name,
                    )*
                    _ => return Err($crate::err::LibcryptErr::InvalidConversion),
                })
            }
        }
    };
}

/// Convert bit flags to and from a struct
macro_rules! bitflags_to_from_struct {
    ( #[$meta:meta] $flags_type:ident, $flag_type:ty, $bitflags_type:ty ) => {
        #[$meta]
        pub struct $flags_type(Vec<$flag_type>);

        impl $flags_type {
            /// Create a new set of flags
            pub fn new(vec: Vec<$flag_type>) -> Self {
                $flags_type(vec)
            }

            /// Create an empty set of flags
            pub fn empty() -> Self {
                $flags_type(Vec::new())
            }
        }

        impl std::convert::Into<$bitflags_type> for $flags_type {
            fn into(self) -> $bitflags_type {
                self.0.into_iter().fold(0, |acc, flag| {
                    let flag: $bitflags_type = flag.into();
                    acc | flag
                })
            }
        }

        impl std::convert::TryFrom<$bitflags_type> for $flags_type {
            type Error = LibcryptErr;

            fn try_from(v: $bitflags_type) -> Result<Self, Self::Error> {
                let mut vec = vec![];
                for i in 0..std::mem::size_of::<$bitflags_type>() * 8 {
                    if (v & (1 << i)) == (1 << i) {
                        vec.push(<$flag_type>::try_from(1 << i)?);
                    }
                }
                Ok(<$flags_type>::new(vec))
            }
        }
    };
}

/// Convert bit a struct reference to bitflags
macro_rules! struct_ref_to_bitflags {
    ( $flags_type:ident, $flag_type:ty, $bitflags_type:ty ) => {
        impl<'a> std::convert::Into<$bitflags_type> for &'a $flags_type {
            fn into(self) -> $bitflags_type {
                self.0.iter().fold(0, |acc, flag| {
                    let flag: $bitflags_type = (*flag).into();
                    acc | flag
                })
            }
        }
    };
}

#[macro_export]
/// Create a C-compatible static string with a null byte
macro_rules! c_str {
    ( $str:tt ) => {
        concat!($str, "\0")
    };
}

#[macro_export]
/// Create a C-compatible callback to determine user confirmation which wraps safe Rust code
macro_rules! c_confirm_callback {
    ( $fn_name:ident, $type:ty, $safe_fn_name:ident ) => {
        extern "C" fn $fn_name(
            msg: *const std::os::raw::c_char,
            usrptr: *mut std::os::raw::c_void,
        ) -> std::os::raw::c_int {
            let msg_str =
                from_str_ptr!(msg).expect("Invalid message string passed to cryptsetup-rs");
            let generic_ptr = usrptr as *mut $type;
            let generic_ref = unsafe { generic_ptr.as_mut() };

            $safe_fn_name(msg_str, generic_ref) as std::os::raw::c_int
        }
    };
}

#[macro_export]
/// Create a C-compatible logging callback which wraps safe Rust code
macro_rules! c_logging_callback {
    ( $fn_name:ident, $type:ty, $safe_fn_name:ident ) => {
        extern "C" fn $fn_name(
            level: std::os::raw::c_int,
            msg: *const std::os::raw::c_char,
            usrptr: *mut std::os::raw::c_void,
        ) {
            let level =
                <$crate::CryptLogLevel as std::convert::TryFrom<std::os::raw::c_int>>::try_from(
                    level,
                )
                .expect("Invalid logging level passed to cryptsetup-rs");
            let msg_str =
                from_str_ptr!(msg).expect("Invalid message string passed to cryptsetup-rs");
            let generic_ptr = usrptr as *mut $type;
            let generic_ref = unsafe { generic_ptr.as_mut() };

            $safe_fn_name(level, msg_str, generic_ref);
        }
    };
}

#[macro_export]
/// Create a C-compatible progress callback for wiping a device which wraps safe Rust code
macro_rules! c_progress_callback {
    ( $fn_name:ident, $type:ty, $safe_fn_name:ident ) => {
        extern "C" fn $fn_name(
            size: u64,
            offset: u64,
            usrptr: *mut std::os::raw::c_void,
        ) -> std::os::raw::c_int {
            let generic_ptr = usrptr as *mut $type;
            let generic_ref = unsafe { generic_ptr.as_mut() };

            $safe_fn_name(size, offset, generic_ref) as std::os::raw::c_int
        }
    };
}

#[macro_export]
/// Create a C-compatible open callback compatible with `CryptTokenHandler`
macro_rules! c_token_handler_open {
    ( $fn_name:ident, $type:ty, $safe_fn_name:ident ) => {
        extern "C" fn $fn_name(
            cd: *mut libcryptsetup_rs_sys::crypt_device,
            token_id: std::os::raw::c_int,
            buffer: *mut *mut std::os::raw::c_char,
            buffer_len: *mut $crate::SizeT,
            usrptr: *mut std::os::raw::c_void,
        ) -> std::os::raw::c_int {
            let device = $crate::device::CryptDevice::from_ptr(cd);
            let generic_ptr = usrptr as *mut $type;
            let generic_ref = unsafe { generic_ptr.as_mut() };

            let buffer: Result<Box<[u8]>, $crate::err::LibcryptErr> =
                $safe_fn_name(device, token_id, generic_ref);
            match buffer {
                Ok(()) => {
                    *buffer = Box::into_raw(buffer) as *mut std::os::raw::c_char;
                    0
                }
                Err(_) => -1,
            }
        }
    };
}

#[macro_export]
/// Create a C-compatible callback for free compatible with `CryptTokenHandler`
macro_rules! c_token_handler_free {
    ( $fn_name:ident, $safe_fn_name:ident ) => {
        extern "C" fn $fn_name(buffer: *mut std::os::raw::c_void, buffer_len: $crate::SizeT) {
            let boxed_slice = unsafe {
                Box::from_raw(std::slice::from_raw_parts_mut(
                    buffer as *mut u8,
                    buffer_len as usize,
                ))
            };

            $safe_fn_name(boxed_slice)
        }
    };
}

#[macro_export]
/// Create a C-compatible callback for validate compatible with `CryptTokenHandler`
macro_rules! c_token_handler_validate {
    ( $fn_name:ident, $safe_fn_name:ident ) => {
        extern "C" fn $fn_name(
            cd: *mut libcryptsetup_rs_sys::crypt_device,
            json: *mut std::os::raw::c_char,
        ) -> std::os::raw::c_int {
            let device = $crate::device::CryptDevice::from_ptr(cd);
            let s = match from_str_ptr!(json) {
                Ok(s) => s,
                Err(_) => return -1,
            };
            let json_obj = match serde_json::from_str(s) {
                Ok(j) => j,
                Err(_) => return -1,
            };

            let rc: Result<(), $crate::err::LibcryptErr> = $safe_fn_name(device, json_obj);
            match rc {
                Ok(()) => 0,
                Err(_) => -1,
            }
        }
    };
}

#[macro_export]
/// Create a C-compatible callback for compatible with `CryptTokenHandler`
macro_rules! c_token_handler_dump {
    ( $fn_name:ident, $safe_fn_name:ident ) => {
        extern "C" fn $fn_name(
            cd: *mut libcryptsetup_rs_sys::crypt_device,
            json: *mut std::os::raw::c_char,
        ) {
            let device = $crate::device::CryptDevice::from_ptr(cd);
            let s = match from_str_ptr!(json) {
                Ok(s) => s,
                Err(_) => return,
            };
            let json_obj = match serde_json::from_str(s) {
                Ok(j) => j,
                Err(_) => return,
            };

            $safe_fn_name(device, json_obj)
        }
    };
}

#[cfg(test)]
mod test {
    use crate::{log::CryptLogLevel, Bool, Interrupt};

    fn safe_confirm_callback(_msg: &str, usrdata: Option<&mut u64>) -> Bool {
        Bool::from(*usrdata.unwrap() as i32)
    }

    c_confirm_callback!(confirm_callback, u64, safe_confirm_callback);

    fn safe_logging_callback(_level: CryptLogLevel, _msg: &str, _usrdata: Option<&mut u64>) {}

    c_logging_callback!(logging_callback, u64, safe_logging_callback);

    fn safe_progress_callback(_size: u64, _offset: u64, usrdata: Option<&mut u64>) -> Interrupt {
        Interrupt::from(*usrdata.unwrap() as i32)
    }

    c_progress_callback!(progress_callback, u64, safe_progress_callback);

    #[test]
    fn test_c_confirm_callback() {
        let ret = confirm_callback(
            "".as_ptr() as *const std::os::raw::c_char,
            &mut 1 as *mut _ as *mut std::os::raw::c_void,
        );
        assert_eq!(1, ret);
        assert_eq!(Bool::Yes, Bool::from(ret));

        let ret = confirm_callback(
            "".as_ptr() as *const std::os::raw::c_char,
            &mut 0 as *mut _ as *mut std::os::raw::c_void,
        );
        assert_eq!(0, ret);
        assert_eq!(Bool::No, Bool::from(ret));
    }

    #[test]
    fn test_c_logging_callback() {
        logging_callback(
            libcryptsetup_rs_sys::CRYPT_LOG_ERROR as i32,
            "".as_ptr() as *const std::os::raw::c_char,
            &mut 1 as *mut _ as *mut std::os::raw::c_void,
        );

        logging_callback(
            libcryptsetup_rs_sys::CRYPT_LOG_DEBUG as i32,
            "".as_ptr() as *const std::os::raw::c_char,
            &mut 0 as *mut _ as *mut std::os::raw::c_void,
        );
    }

    #[test]
    fn test_c_progress_callback() {
        let ret = progress_callback(0, 0, &mut 1 as *mut _ as *mut std::os::raw::c_void);
        assert_eq!(1, ret);
        assert_eq!(Interrupt::Yes, Interrupt::from(ret));

        let ret = progress_callback(0, 0, &mut 0 as *mut _ as *mut std::os::raw::c_void);
        assert_eq!(0, ret);
        assert_eq!(Interrupt::No, Interrupt::from(ret));
    }
}
