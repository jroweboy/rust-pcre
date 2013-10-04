// Copyright 2013 The rust-pcre authors.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::c_str::*;
use std::libc::*;
use std::ptr;

pub type fullinfo_field = c_int;
pub struct pcre;
pub type pcre_error = c_int;
pub struct pcre_extra;

pub static PCRE_ERROR_NOMATCH: pcre_error = -1;
pub static PCRE_ERROR_NULL: pcre_error = -2;

pub static PCRE_INFO_CAPTURECOUNT: fullinfo_field = 2;

mod native;

#[fixed_stack_segment]
#[inline(never)]
pub fn pcre_compile(pattern: *c_char, options: ::options, tableptr: *c_uchar) -> *pcre {
    assert!(ptr::is_not_null(pattern));
    let mut err: *c_char = ptr::null();
    let mut erroffset: c_int = 0;
    let code = unsafe { native::pcre_compile(pattern, options, &mut err, &mut erroffset, tableptr) };

    if ptr::is_null(code) {
        // "Otherwise, if  compilation  of  a  pattern fails, pcre_compile() returns
        // NULL, and sets the variable pointed to by errptr to point to a textual
        // error message. This is a static string that is part of the library. You
        // must not try to free it."
        // http://pcre.org/pcre.txt
        let err_cstring = unsafe { CString::new(err, false) };
        match err_cstring.as_str() {
            None          => error!("pcre_compile() failed at offset %u", erroffset as uint),
            Some(err_str) => error!("pcre_compile() failed at offset %u: %s", erroffset as uint, err_str)
        }
        fail!("pcre_compile");
    }
    assert!(ptr::is_not_null(code));
    assert_eq!(erroffset, 0);

    code
}

#[fixed_stack_segment]
#[inline(never)]
pub fn pcre_exec(code: *pcre, extra: *pcre_extra, subject: *c_char, length: c_int, startoffset: c_int, options: ::options, ovector: *mut c_int, ovecsize: c_int) -> bool {
    assert!(ptr::is_not_null(code));
    assert!(ovecsize >= 0 && ovecsize % 3 == 0);
    let rc = unsafe { native::pcre_exec(code, extra, subject, length, startoffset, options, ovector, ovecsize) };
    if rc == PCRE_ERROR_NOMATCH {
        return false;
    } else if rc < 0 && rc != PCRE_ERROR_NULL {
        fail!("pcre_exec");
    }

    true
}

#[fixed_stack_segment]
#[inline(never)]
pub fn pcre_free(ptr: *c_void) {
    native::pcre_free(ptr);
}

#[fixed_stack_segment]
#[inline(never)]
pub fn pcre_free_study(extra: *pcre_extra) {
    unsafe { native::pcre_free_study(extra); }
}

#[fixed_stack_segment]
#[inline(never)]
pub fn pcre_fullinfo(code: *pcre, extra: *pcre_extra, what: fullinfo_field, where: *mut c_void) {
    assert!(ptr::is_not_null(code));
    let rc = unsafe { native::pcre_fullinfo(code, extra, what, where) };
    if rc < 0 && rc != PCRE_ERROR_NULL {
        fail!("pcre_fullinfo");
    }
}

#[fixed_stack_segment]
#[inline(never)]
pub fn pcre_study(code: *::detail::pcre, options: ::study_options) -> *::detail::pcre_extra {
    assert!(ptr::is_not_null(code));
    let mut err: *c_char = ptr::null();
    let extra = unsafe { native::pcre_study(code, options, &mut err) };
    // "The third argument for pcre_study() is a pointer for an error message. If
    // studying succeeds (even if no data is returned), the variable it points to is
    // set to NULL. Otherwise it is set to point to a textual error message. This is
    // a static string that is part of the library. You must not try to free it."
    // http://pcre.org/pcre.txt
    if ptr::is_not_null(err) {
        let err_cstring = unsafe { CString::new(err, false) };
        match err_cstring.as_str() {
            None          => error!("pcre_study() failed"),
            Some(err_str) => error!("pcre_study() failed: %s", err_str)
        }
        fail!("pcre_study");
    }
    assert!(ptr::is_null(err));

    extra
}