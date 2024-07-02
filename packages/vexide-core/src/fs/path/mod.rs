mod fs_str;
use fs_str::FsStr;

#[repr(transparent)]
pub struct Path {
    pub(crate) inner: FsStr,
}
impl Path {
    pub fn new<'a, P: AsRef<FsStr> + 'a>(path: P) -> &'a Self {
        unsafe { &*(path.as_ref() as *const FsStr as *const Path) }
    }
}
impl AsRef<Path> for Path {
    fn as_ref(&self) -> &Path {
        self
    }
}
