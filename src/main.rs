use windows::{
    core::*, Win32::Foundation::*, Win32::Storage::FileSystem::*, Win32::System::SystemServices::*,
    Win32::System::IO::*,
};

fn main() -> Result<()> {
    unsafe {
        let h_file = CreateFileA(
            "\\\\.\\Global\\{4B1E624A-2DEA-4205-8F5F-596A45BECF24}.tap",
            GENERIC_READ | GENERIC_WRITE,
            0,
            std::ptr::null_mut(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_SYSTEM,
            HANDLE(0)
        );
        println!("{}", h_file.is_invalid());
    }
    println!("Hello, world!");
    Ok(())
}
