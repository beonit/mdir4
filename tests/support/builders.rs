use std::path::Path;

use mdir4::{
    adapters::memory_fs::{MemoryFileSystem, MemoryFileSystemBuilder},
    ports::filesystem::FsOperation,
};

pub const ROOT: &str = r"C:\WORK";

pub fn empty_filesystem() -> MemoryFileSystem {
    MemoryFileSystemBuilder::new().directory(ROOT).build()
}

pub fn basic_filesystem() -> MemoryFileSystem {
    MemoryFileSystemBuilder::new()
        .directory(ROOT)
        .file(r"C:\WORK\README.TXT", 128)
        .file(r"C:\WORK\TEST.EXE", 4_096)
        .file(r"C:\WORK\CONFIG.INI", 64)
        .directory(r"C:\WORK\SUBDIR")
        .build()
}

pub fn unicode_filesystem() -> MemoryFileSystem {
    MemoryFileSystemBuilder::new()
        .directory(ROOT)
        .file(r"C:\WORK\한글.txt", 10)
        .file(r"C:\WORK\日本語.txt", 20)
        .file(r"C:\WORK\é.txt", 30)
        .file(r"C:\WORK\📁.txt", 40)
        .build()
}

pub fn nested_filesystem() -> MemoryFileSystem {
    MemoryFileSystemBuilder::new()
        .directory(ROOT)
        .directory(r"C:\WORK\ONE")
        .directory(r"C:\WORK\ONE\TWO")
        .directory(r"C:\WORK\ONE\TWO\THREE")
        .directory(r"C:\WORK\ONE\TWO\THREE\FOUR")
        .directory(r"C:\WORK\ONE\TWO\THREE\FOUR\FIVE")
        .file(r"C:\WORK\ONE\TWO\THREE\FOUR\FIVE\END.TXT", 5)
        .build()
}

pub fn permission_denied_filesystem() -> MemoryFileSystem {
    MemoryFileSystemBuilder::new()
        .directory(ROOT)
        .directory(r"C:\WORK\SECRET")
        .file(r"C:\WORK\LOCKED.TXT", 99)
        .deny(FsOperation::ReadDirectory, Path::new(r"C:\WORK\SECRET"))
        .deny(FsOperation::ReadMetadata, Path::new(r"C:\WORK\LOCKED.TXT"))
        .build()
}
