use std::path::Path;
use std::sync::Arc;

#[derive(Clone)]
pub(super) struct PackageBytes {
    inner: Arc<PackageBytesInner>,
}

enum PackageBytesInner {
    #[cfg(target_os = "linux")]
    Mapped(MappedFile),
    Owned(Box<[u8]>),
}

impl PackageBytes {
    pub(super) fn load(path: &Path) -> Result<Self, String> {
        #[cfg(target_os = "linux")]
        {
            MappedFile::open(path).map(|mapped| Self {
                inner: Arc::new(PackageBytesInner::Mapped(mapped)),
            })
        }
        #[cfg(not(target_os = "linux"))]
        {
            std::fs::read(path)
                .map(Self::from_vec)
                .map_err(|error| format!("{}: {error}", path.display()))
        }
    }

    pub(super) fn from_vec(bytes: Vec<u8>) -> Self {
        Self {
            inner: Arc::new(PackageBytesInner::Owned(bytes.into_boxed_slice())),
        }
    }

    pub(super) fn as_slice(&self) -> &[u8] {
        match self.inner.as_ref() {
            #[cfg(target_os = "linux")]
            PackageBytesInner::Mapped(mapped) => mapped.as_slice(),
            PackageBytesInner::Owned(bytes) => bytes,
        }
    }

    pub(super) fn len(&self) -> usize {
        self.as_slice().len()
    }

    pub(super) fn is_mapped(&self) -> bool {
        #[cfg(target_os = "linux")]
        {
            matches!(self.inner.as_ref(), PackageBytesInner::Mapped(_))
        }
        #[cfg(not(target_os = "linux"))]
        {
            false
        }
    }
}

impl std::fmt::Debug for PackageBytes {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PackageBytes")
            .field("len", &self.len())
            .field("mapped", &self.is_mapped())
            .finish()
    }
}

#[cfg(target_os = "linux")]
struct MappedFile {
    ptr: *mut libc::c_void,
    len: usize,
}

#[cfg(target_os = "linux")]
impl MappedFile {
    fn open(path: &Path) -> Result<Self, String> {
        use std::fs::File;
        use std::os::fd::AsRawFd;

        let file = File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
        let file_len = file
            .metadata()
            .map_err(|error| format!("{}: {error}", path.display()))?
            .len();
        let len = usize::try_from(file_len)
            .map_err(|_| format!("{}: L2 package exceeds address space", path.display()))?;
        if len == 0 {
            return Err(format!("{}: empty L2 package", path.display()));
        }
        let ptr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                len,
                libc::PROT_READ,
                libc::MAP_PRIVATE,
                file.as_raw_fd(),
                0,
            )
        };
        if ptr == libc::MAP_FAILED {
            return Err(format!(
                "{}: mmap failed: {}",
                path.display(),
                std::io::Error::last_os_error()
            ));
        }
        Ok(Self { ptr, len })
    }

    fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr.cast::<u8>(), self.len) }
    }
}

#[cfg(target_os = "linux")]
unsafe impl Send for MappedFile {}
#[cfg(target_os = "linux")]
unsafe impl Sync for MappedFile {}

#[cfg(target_os = "linux")]
impl Drop for MappedFile {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.ptr, self.len);
        }
    }
}
