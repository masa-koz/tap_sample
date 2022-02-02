use futures::future;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::os::windows::prelude::*;
use std::thread;
use std::time::Duration;
use tokio::fs::File;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt};
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
        let icmp_echo: [u8; 74] = [
        0x00, 0xff, 0x4b, 0x1e, 0x62, 0x4a, 0xfc, 0x34, 0x97, 0x97, 0x4f, 0xed, 0x08, 0x00, 0x45,
        0x00, 0x00, 0x3c, 0x34, 0xf0, 0x00, 0x00, 0x80, 0x01, 0x00, 0x00, 0xac, 0x11, 0xff, 0x02,
        0xac, 0x11, 0xff, 0x01, 0x08, 0x00, 0x4d, 0x2b, 0x00, 0x01, 0x00, 0x30, 0x61, 0x62, 0x63,
        0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6a, 0x6b, 0x6c, 0x6d, 0x6e, 0x6f, 0x70, 0x71, 0x72,
        0x73, 0x74, 0x75, 0x76, 0x77, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69,
    ];

    let mut packet: [u8; 4096] = [0; 4096];
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(false)
        .attributes(FILE_ATTRIBUTE_SYSTEM | FILE_FLAG_OVERLAPPED)
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

    /*
    let mut named_pipe = unsafe { mio::windows::NamedPipe::from_raw_handle(file.as_raw_handle()) };
    let mut poll = mio::Poll::new().unwrap();
    poll.registry()
        .register(&mut named_pipe, mio::Token(0), mio::Interest::READABLE)
        .unwrap();
    let mut events = mio::Events::with_capacity(1024);
    loop {
        poll.poll(&mut events, None).unwrap();
        for event in events.iter() {
            match event.token() {
                mio::Token(..) => {
                    let ret = named_pipe.read(&mut packet[..]);
                    println!("ret: {:?}", ret);
                    if let Ok(n) = ret {
                        println!("The bytes: {:?}", &packet[..n]);
                    }
                }
            }
        }
    }
    */

    let cpus = num_cpus::get();
    println!("logical cores: {}", cpus);

    use tokio::sync::mpsc;
    let (tx_a, mut rx_a) = mpsc::channel::<String>(10);
    let (tx_b, mut rx_b) = mpsc::channel::<String>(10);

    let mut handles = Vec::new();

    let mut named_pipe = Arc::new(unsafe {
        tokio::net::windows::named_pipe::NamedPipeClient::from_raw_handle(
            file.as_raw_handle(),
        )
        .unwrap()
    });

    let file1 = named_pipe.clone();
    let task_a = tokio::task::Builder::new()
        .name("Task A")
        .spawn(async move {
            /*
            let mut file = File::from_std(file);
            loop {
                let n = file.read(&mut packet[..]).await.unwrap();
                println!("The bytes: {:?}", &packet[..n]);
            }
            */
            loop {
                tokio::select! {
                    _ = file1.readable() => {
                        match file1.try_read(&mut packet[..]) {
                            Ok(n) => {                      
                                tx_a.send(format!("Task A: Read packet {} bytes", n)).await.unwrap();
                            },
                            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {},
                            Err(e) => {
                                tx_a.send(format!("Task A: Read failed {:?}", e)).await.unwrap();
                            }
                        }
                    },
                    _ = sleep(Duration::from_millis(68719476734)) => {
                        tx_a.send(format!("Task A: wakeup")).await.unwrap();
                    }
                }
            }
        });
    handles.push(task_a);

    let file2 = named_pipe.clone();
    let task_b = tokio::task::Builder::new()
        .name("Task B")
        .spawn(async move {
            loop {
                thread::sleep(Duration::from_secs(5));
                loop {
                    if file2.writable().await.is_ok() {
                        match file2.try_write(&icmp_echo) {
                            Ok(n) => {
                                tx_b.send(format!("Task B: Write Packet {} bytes.", n)).await.unwrap();
                                break;
                            }
                            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                tx_b.send(format!("Task B: Write would block")).await.unwrap();
                                continue;
                            },
                            Err(e) => {
                                tx_b.send(format!("Task b: Write failed {:?}", e)).await.unwrap();
                                break;
                            }
                        }
                    }
                }
                sleep(Duration::from_nanos(1)).await;

            }
        });
    handles.push(task_b);

    loop {
        tokio::select! {
            val = rx_a.recv() => {
                println!("rx_a: {:?}", val);
            }
            val = rx_b.recv() => {
                println!("rx_b: {:?}", val);
            }
        }
    }

    Ok(())
}
