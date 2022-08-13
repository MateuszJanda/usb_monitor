use futures::executor::block_on;
use std::collections::HashSet;

use windows::core::*;
use windows::Devices::Enumeration::DeviceInformation;
use windows::Win32::{
    Foundation::*, System::LibraryLoader::GetModuleHandleA, System::SystemServices,
    UI::WindowsAndMessaging::*,
};

fn main() -> Result<()> {
    let mut usb_monitor = UsbMonitor::new();
    usb_monitor.run()
}

struct UsbMonitor {
    /// Handler to current window.
    handle: HWND,
    /// Set of all newly plugged USB devices.
    devices: HashSet<String>,
}

impl UsbMonitor {
    fn new() -> Self {
        UsbMonitor {
            handle: HWND(0),
            devices: HashSet::new(),
        }
    }

    /// Register window and run main loop.
    fn run(&mut self) -> Result<()> {
        unsafe {
            let instance = GetModuleHandleA(None)?;
            debug_assert!(instance.0 != 0);

            // Not predefined Windows class name, so will be registered by RegisterClassA.
            let window_class = s!("usb_monitor_window");

            let wc = WNDCLASSA {
                hInstance: instance,
                lpszClassName: window_class,

                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(wndproc),
                ..Default::default()
            };

            let atom = RegisterClassA(&wc);
            debug_assert!(atom != 0);

            // Register window. lpParam is set as pointer to current UsbMonitor.
            let handle = CreateWindowExA(
                WINDOW_EX_STYLE::default(),
                window_class,
                s!("This is a sample window"),
                WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                None,
                None,
                instance,
                self as *mut _ as _,
            );

            debug_assert!(handle.0 != 0);
            debug_assert!(handle == self.handle);

            let mut message = MSG::default();

            while GetMessageA(&mut message, HWND(0), 0, 0).into() {
                DispatchMessageA(&message);
            }

            Ok(())
        }
    }

    /// Handle all window messages, especially WM_DEVICECHANGE when device or piece of media has
    /// been inserted and is now available. It search then for newly plugged USB devices.
    fn message_handler(&mut self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        unsafe {
            match message as u32 {
                WM_CREATE => {
                    println!("WM_CREATE. Archived USB info: {}", self.devices.len());
                    LRESULT(0)
                }
                WM_DESTROY => {
                    println!("WM_DESTROY");
                    PostQuitMessage(0);
                    LRESULT(0)
                }
                WM_DEVICECHANGE => {
                    match wparam.0 as u32 {
                        SystemServices::DBT_DEVICEARRIVAL => {
                            println!("DBT_DEVICEARRIVAL. New device plugged.");

                            let future = get_all_usb_info();
                            let new_devices = block_on(future).unwrap();

                            for device_id in new_devices {
                                if !self.devices.contains(&device_id) {
                                    println!("New USB info: {}", device_id);
                                    self.devices.insert(device_id);
                                }
                            }

                            println!("Archived USB info: {}", self.devices.len());
                        }
                        SystemServices::DBT_DEVICEREMOVECOMPLETE => {
                            println!("DBT_DEVICEARRIVAL. Device unplugged.")
                        }
                        _ => (),
                    }
                    LRESULT(0)
                }

                _ => DefWindowProcA(self.handle, message, wparam, lparam),
            }
        }
    }
}

/// Basic function to processes messages (registered in RegisterClassA), that that pass all messages
/// to UsbMonitor object (message_handler method).
extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        // WM_NCCREATE is called even before WM_CREATE, so this is good place to read and save
        // pointer to UsbMonitor object.
        if message == WM_NCCREATE {
            println!("WM_NCCREATE. Saving pointer to UsbMonitor object.");
            let cs = lparam.0 as *const CREATESTRUCTA;
            let this = (*cs).lpCreateParams as *mut UsbMonitor;

            match this.is_null() {
                false => (*this).handle = window,
                true => panic!("lpCreateParams is set to NULL. Check CreateWindowExA."),
            }

            SetWindowLongPtrA(window, GWLP_USERDATA, this as _);
        } else {
            let this = GetWindowLongPtrA(window, GWLP_USERDATA) as *mut UsbMonitor;

            if !this.is_null() {
                return (*this).message_handler(message, wparam, lparam);
            }
        }

        DefWindowProcA(window, message, wparam, lparam)
    }
}

/// Iterate trough all devices and return raw information about USB devices (DeviceInformation.Id).
/// Function use UWP (Universal Windows Platform) API.
///
/// [Finding the device](https://docs.microsoft.com/en-us/windows-hardware/drivers/usbcon/how-to-connect-to-a-usb-device--uwp-app-#finding-the-devicethe-basic-way)
async fn get_all_usb_info() -> Result<HashSet<String>> {
    let mut result = HashSet::new();

    let dev_info_collection = DeviceInformation::FindAllAsync()?.await?;
    for dev_info in dev_info_collection {
        if let Ok(device_id) = dev_info.Id() {
            if device_id.to_string().contains("\\USB#") {
                result.insert(device_id.to_string());
            }
        }
    }

    Ok(result)
}
