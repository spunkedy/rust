//! This module implements a type which stores a nul-terminated path that has
//! space available for a 32 bit error code from the OS.
//!
//! Conceptually, this is similar to `ThinBox<(i32, [u8])>`, where the `[u8]` is
//! a nul-terminated slice of path bytes. If we add `ThinBox::from_raw` and
//! `ThinBox::into_raw` in the future (which seems likely), we can consider
//! changing the implementation to that, although the `(i32, [u8])` would need
//! to be replaced with something that guarantees 8-byte alignment when used
//! with repr_bitpacked.
// use core::mem::{align_of, size_of};
// use core::marker::PhantomData;
use crate::alloc::{alloc, dealloc, Layout, LayoutError};
use crate::path::Path;
use crate::sys_common::memchr;
use core::ptr::NonNull;

#[repr(C, align(8))]
#[derive(Clone, Copy)]
struct OsPathAndErrorHeader {
    len_with_nul: usize,
    os_code: i32,
    // Note: Data starts at the end of the struct (e.g. after 4 padding bytes)
}

pub struct OsPathBufAndError {
    ptr: NonNull<OsPathAndErrorHeader>,
}

unsafe impl Send for OsPathBufAndError {}
unsafe impl Sync for OsPathBufAndError {}

impl OsPathBufAndError {
    #[cfg(not(no_global_oom_handling))]
    fn from_path_bytes(bytes: &[u8]) -> crate::io::Result<Self> {
        if memchr::memchr(0, bytes).is_some() {
            return Err(super::const_io_error!(
                super::ErrorKind::InvalidInput,
                "path contains interior nul byte",
            ));
        }
        let len_with_nul = bytes.len() + 1;
        let full_layout = if let Ok(full_layout) = layout_for_len(len_with_nul) {
            debug_assert!(full_layout.size() != 0);
            full_layout
        } else {
            // We don't have a layout to give to `handle_alloc_error`, but this
            // case is basically impossible anyway.
            return Err(super::const_io_error!(
                super::ErrorKind::OutOfMemory,
                "cannot allocate memory for nul terminated path",
            ));
        };
        unsafe {
            let p = crate::alloc::alloc(full_layout);
            if p.is_null() {
                crate::alloc::handle_alloc_error(full_layout);
            }
            let header_ptr = p.cast::<OsPathAndErrorHeader>();
            // Write the header
            header_ptr.write(OsPathAndErrorHeader { os_code: 0, len_with_nul });
            // Write the path
            let path_ptr = header_ptr.add(1).cast::<u8>(); //p.add(core::mem::size_of::<OsPathAndErrorHeader>());
            path_ptr.copy_from_nonoverlapping(bytes.as_ptr(), bytes.len());
            // Write the nul-terminator.
            path_ptr.add(bytes.len()).write(0);
            Ok(Self { ptr: NonNull::new_unchecked(header_ptr) })
        }
    }

    #[inline]
    pub(crate) fn code(&self) -> i32 {
        unsafe { core::ptr::addr_of!((*self.ptr.as_ptr()).os_code).read() }
    }

    #[inline]
    pub(crate) fn set_code(&mut self, code: i32) {
        unsafe { core::ptr::addr_of_mut!((*self.ptr.as_ptr()).os_code).write(code) }
    }

    #[inline]
    pub(crate) fn into_raw(self) -> NonNull<u8> {
        let me = core::mem::ManuallyDrop::new(self);
        me.ptr.cast()
    }

    #[inline]
    pub(crate) unsafe fn from_raw(p: NonNull<u8>) -> Self {
        Self { ptr: p.cast() }
    }

    #[inline]
    pub(crate) fn as_cstr(&self) -> &crate::ffi::CStr {
        unsafe {
            let hdr = self.ptr.as_ptr();
            let len_with_nul = core::ptr::addr_of!((*hdr).len_with_nul).read();
            let path_start = hdr.add(1).cast::<u8>();
            let slice = core::slice::from_raw_parts(path_start, len_with_nul);
            crate::ffi::CStr::from_bytes_with_nul_unchecked(slice)
        }
    }

    #[inline]
    pub(crate) unsafe fn code_from_raw(ptr: NonNull<u8>) -> i32 {
        unsafe {
            let hptr = ptr.as_ptr().cast::<OsPathAndErrorHeader>();
            core::ptr::addr_of!((*hptr).os_code).read()
        }
    }

    #[inline]
    pub(crate) unsafe fn cstr_from_raw<'a>(ptr: NonNull<u8>) -> &'a crate::ffi::CStr {
        let hdr = ptr.cast::<OsPathAndErrorHeader>().as_ptr();
        let len_with_nul = core::ptr::addr_of!((*hdr).len_with_nul).read();
        let path_start = hdr.add(1).cast::<u8>();
        let slice = core::slice::from_raw_parts(path_start, len_with_nul);
        crate::ffi::CStr::from_bytes_with_nul_unchecked(slice)
    }
}

impl Drop for OsPathBufAndError {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            let header = self.ptr.as_ptr();
            let len_with_nul = core::ptr::addr_of!((*header).len_with_nul).read();
            let layout = layout_for_len(len_with_nul).unwrap_unchecked();
            crate::alloc::dealloc(header.cast(), layout);
        }
    }
}

#[inline]
fn layout_for_len(len: usize) -> Result<Layout, LayoutError> {
    let header_layout = Layout::new::<OsPathAndErrorHeader>();
    let path_layout = Layout::array::<u8>(len)?;
    let (layout, offset) = header_layout.extend(path_layout)?;
    debug_assert_eq!(offset, core::mem::size_of::<OsPathAndErrorHeader>());
    Ok(layout)
}
