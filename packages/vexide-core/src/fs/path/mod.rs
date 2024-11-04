mod fs_str;
use fs_str::FsStr;

#[repr(transparent)]
pub struct Path {
    pub(crate) inner: FsStr,
}
impl Path {
    pub fn new<'a, P: AsRef<FsStr> + 'a>(path: P) -> &'a Self {
        unsafe { &*(core::ptr::from_ref::<FsStr>(path.as_ref()) as *const Path) }
    }
    pub fn as_os_str(&self) -> &FsStr {
        &self.inner
    }
}
impl AsRef<Path> for Path {
    fn as_ref(&self) -> &Path {
        self
    }
}
