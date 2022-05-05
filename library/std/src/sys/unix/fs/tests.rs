use super::with_cstr;
use crate::ffi::{CStr, CString};
use crate::io;
use crate::os::unix::ffi::OsStrExt;
use crate::path::Path;

#[test]
fn test_with_cstr() {
    #[track_caller]
    fn check(p: &Path) {
        let expect = super::cstr(p);
        let got: io::Result<CString> = with_cstr(p, |c_p: &CStr| {
            assert_eq!(c_p.to_str().unwrap().as_bytes(), p.as_os_str().as_bytes(),);
            Ok(c_p.to_owned())
        });

        match (expect, got) {
            (Ok(s1), Ok(s2)) => assert_eq!(s1, s2),
            (Err(e1), Err(e2)) => assert_eq!(e1.kind(), e2.kind()),
            (r0, r1) => {
                panic!("Bad result for {r0:?} and {r1:?} seem different (input was `{p:?}`)")
            }
        }
    }

    let buffer = ('a'..'z').cycle().take(1024).collect::<String>();

    for i in 0..buffer.len() {
        let p: &str = &buffer[..i];
        check(p.as_ref());
        let mut tmp = p.as_bytes().to_vec();
        for j in 0..tmp.len() {
            let old = core::mem::replace(&mut tmp[j], 0);
            check(core::str::from_utf8(&tmp).unwrap().as_ref());
            tmp[j] = old;
        }
    }
}
