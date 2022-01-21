use futures::future;
use std::fs::OpenOptions;
use std::os::windows::prelude::*;
use std::thread;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::time::sleep;
use windows::{
    core::*, Win32::Foundation::*, Win32::Storage::FileSystem::*, Win32::System::SystemServices::*,
    Win32::System::Threading::*, Win32::System::IO::*,
};

const TAP_WIN_IOCTL_GET_MAC: u32 = 0x00000022 << 16 | 0 << 14 | 1 << 2 | 0;
const TAP_WIN_IOCTL_GET_VERSION: u32 = 0x00000022 << 16 | 0 << 14 | 2 << 2 | 0;
const TAP_WIN_IOCTL_GET_MTU: u32 = 0x00000022 << 16 | 0 << 14 | 3 << 2 | 0;
const TAP_WIN_IOCTL_GET_INFO: u32 = 0x00000022 << 16 | 0 << 14 | 4 << 2 | 0;
const TAP_WIN_IOCTL_CONFIG_POINT_TO_POINT: u32 = 0x00000022 << 16 | 0 << 14 | 5 << 2 | 0;
const TAP_WIN_IOCTL_SET_MEDIA_STATUS: u32 = 0x00000022 << 16 | 0 << 14 | 6 << 2 | 0;
const TAP_WIN_IOCTL_CONFIG_DHCP_MASQ: u32 = 0x00000022 << 16 | 0 << 14 | 7 << 2 | 0;
const TAP_WIN_IOCTL_GET_LOG_LINE: u32 = 0x00000022 << 16 | 0 << 14 | 8 << 2 | 0;
const TAP_WIN_IOCTL_CONFIG_DHCP_SET_OPT: u32 = 0x00000022 << 16 | 0 << 14 | 9 << 2 | 0;

#[tokio::main]
async fn main() -> Result<()> {
    console_subscriber::init();

    let mut packet: [u8; 4096] = [0; 4096];
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(false)
        .attributes(FILE_ATTRIBUTE_SYSTEM)
        .open("\\\\.\\Global\\{4B1E624A-2DEA-4205-8F5F-596A45BECF24}.tap")
        .unwrap();
    unsafe {
        let mut info: [u8; 6] = [0; 6];
        let mut len: u32 = 0;
        if DeviceIoControl(
            HANDLE(file.as_raw_handle() as isize),
            TAP_WIN_IOCTL_GET_MAC,
            std::ptr::null_mut(),
            0,
            info.as_mut_ptr() as _,
            6,
            &mut len,
            std::ptr::null_mut(),
        )
        .as_bool()
        {
            println!(
                "MAC: {:x}-{:x}-{:x}-{:x}-{:x}-{:x}",
                info[0], info[1], info[2], info[3], info[4], info[5],
            );
        }

        let mut info: [u32; 3] = [0; 3];
        let mut len: u32 = 0;
        if DeviceIoControl(
            HANDLE(file.as_raw_handle() as isize),
            TAP_WIN_IOCTL_GET_VERSION,
            std::ptr::null_mut(),
            0,
            info.as_mut_ptr() as _,
            12,
            &mut len,
            std::ptr::null_mut(),
        )
        .as_bool()
        {
            println!(
                "Version: {}.{} {}",
                info[0],
                info[1],
                if info[2] != 0 { "(Debug)" } else { "" }
            );
        }

        let mut info: [u32; 1] = [0; 1];
        let mut len: u32 = 0;
        if DeviceIoControl(
            HANDLE(file.as_raw_handle() as isize),
            TAP_WIN_IOCTL_GET_MTU,
            std::ptr::null_mut(),
            0,
            info.as_mut_ptr() as _,
            4,
            &mut len,
            std::ptr::null_mut(),
        )
        .as_bool()
        {
            println!("MTU: {}", info[0]);
        }

        let mut info: [u8; 256] = [0; 256];
        let mut len: u32 = 0;
        if DeviceIoControl(
            HANDLE(file.as_raw_handle() as isize),
            TAP_WIN_IOCTL_GET_INFO,
            std::ptr::null_mut(),
            0,
            info.as_mut_ptr() as _,
            256,
            &mut len,
            std::ptr::null_mut(),
        )
        .as_bool()
        {
            let info = String::from_utf8_lossy(&info[..len as usize]);
            println!("Info: {}", info);
        }
        let mut info: [u32; 1] = [1; 1];
        let mut len: u32 = 0;
        if DeviceIoControl(
            HANDLE(file.as_raw_handle() as isize),
            TAP_WIN_IOCTL_SET_MEDIA_STATUS,
            info.as_mut_ptr() as _,
            4,
            info.as_mut_ptr() as _,
            4,
            &mut len,
            std::ptr::null_mut(),
        )
        .as_bool()
        {
            let mut len: u32 = 0;

            let mut overlapped = OVERLAPPED {
                Anonymous: OVERLAPPED_0 {
                    Anonymous: OVERLAPPED_0_0 {
                        Offset: 9,
                        OffsetHigh: 0,
                    },
                },
                hEvent: CreateEventA(std::ptr::null_mut(), true, false, None),
                Internal: 0,
                InternalHigh: 0,
            };
            overlapped.hEvent.ok()?;

            /* loop {
                let read_ok = ReadFile(
                    HANDLE(file.as_raw_handle() as isize),
                    packet.as_mut_ptr() as _,
                    4096,
                    &mut len,
                    &mut overlapped,
                );
                if !read_ok.as_bool() {
                    assert_eq!(GetLastError(), ERROR_IO_PENDING);
                }
                let wait_ok = WaitForSingleObject(overlapped.hEvent, 2000);
                assert!(wait_ok == WAIT_OBJECT_0);
                let mut bytes_copied = 0;
                let overlapped_ok = GetOverlappedResult(
                    HANDLE(file.as_raw_handle() as isize),
                    &mut overlapped,
                    &mut bytes_copied,
                    false,
                );
                assert!(overlapped_ok.as_bool());
                println!("bytes_copied: {}", bytes_copied);
            }*/
        }
    }

    let cpus = num_cpus::get();
    println!("logical cores: {}", cpus);

    let mut handles = Vec::new();

    let task_a = tokio::task::Builder::new()
        .name("Task A")
        .spawn(async move {
            let mut file = File::from_std(file);
            loop {
                let n = file.read(&mut packet[..]).await.unwrap();
                println!("The bytes: {:?}", &packet[..n]);
            }
        });
    handles.push(task_a);

    for i in 0..(cpus - 1) + 1 {
        let task_b = tokio::task::Builder::new()
            .name(&format!("Task B-{}", i))
            .spawn(async move {
                loop {
                    println!("#Task B-{} sleeping... {:?}", i, thread::current().id());
                    thread::sleep(Duration::from_secs(5)); // ブロッキング・スリープ
                    println!("#Task B-{} woke up.", i);
                    sleep(Duration::from_nanos(1)).await;
                }
            });
        handles.push(task_b)
    }

    future::join_all(handles).await;

    Ok(())
}
