use futures::executor::block_on;
use std::collections::HashSet;

use windows::core::*;
use windows::Devices::Enumeration::DeviceInformation;
use windows::Win32::{
    Foundation::*, Graphics::Gdi::ValidateRect, System::LibraryLoader::GetModuleHandleA,
    System::SystemServices, UI::WindowsAndMessaging::*,
};

fn main() -> Result<()> {
    let mut usb_monitor = UsbMonitor::new();
    usb_monitor.run()
}

struct UsbMonitor {
    handle: HWND,
    devices: HashSet<String>,
}

impl UsbMonitor {
    fn new() -> Self {
        UsbMonitor {
            handle: HWND(0),
            devices: HashSet::new(),
        }
    }

    fn run(&mut self) -> Result<()> {
        unsafe {
            let instance = GetModuleHandleA(None)?;
            debug_assert!(instance.0 != 0);

            // Not predefined Windows class name, so will be registered by RegisterClassA
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

            // Register window. lpparam is set as pointer to current UsbMonitor
            let handle = CreateWindowExA(
                WINDOW_EX_STYLE::default(),
                window_class,
                s!("This is a sample window"),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
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

    fn message_handler(&mut self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        unsafe {
            match message as u32 {
                WM_CREATE => {
                    println!("WM_CREATE, archived devices: {}", self.devices.len());
                    LRESULT(0)
                }

                WM_PAINT => {
                    println!("WM_PAINT");
                    ValidateRect(self.handle, std::ptr::null());
                    LRESULT(0)
                }
                WM_DESTROY => {
                    println!("WM_DESTROY");
                    PostQuitMessage(0);
                    LRESULT(0)
                }
                WM_DEVICECHANGE => {
                    if wparam.0 as u32 == SystemServices::DBT_DEVICEARRIVAL {
                        println!("DBT_DEVICEARRIVAL ");

                        let future = get_all_usb_info();
                        let new_devices = block_on(future);

                        for device_id in new_devices {
                            if !self.devices.contains(&device_id) {
                                println!("New device: {}", device_id);
                                self.devices.insert(device_id);
                            }
                        }
                    }

                    LRESULT(0)
                }

                _ => DefWindowProcA(self.handle, message, wparam, lparam),
            }
        }
    }
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        if message == WM_NCCREATE {
            println!("WM_NCCREATE");
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

// https://docs.microsoft.com/en-us/windows-hardware/drivers/usbcon/how-to-connect-to-a-usb-device--uwp-app-#finding-the-devicethe-basic-way
async fn get_all_usb_info() -> HashSet<String> {
    let mut result = HashSet::new();

    if let Ok(operation) = DeviceInformation::FindAllAsync() {
        if let Ok(dev_info_collection) = operation.await {
            for dev_info in dev_info_collection {
                if let Ok(device_id) = dev_info.Id() {
                    // if let Some(value) = usb_product_and_vendor(&device_id).await {

                    //     println!("Device: {}-{} [{}]", value.0, value.1, device_id);

                    //     result.insert(device_id.to_string(), value);
                    // }
                    // else
                    // {
                    //     println!("Tutaj1");
                    // }

                    if device_id.to_string().contains("\\USB#") {
                        // result.insert(device_id.to_string(), (0,0));
                        result.insert(device_id.to_string());
                    }

                    if let Ok(a) = dev_info.IsEnabled() {
                        if a {

                            // println!("{}", device_id);
                        } else {
                            // println!("False");
                        }
                    } else {
                        // println!("Not enabled");
                    }
                }
            }
        }
    }

    result
}

// async fn usb_product_and_vendor(device_id: &HSTRING) -> Option<(u32, u32)> {
//     if let Ok(operation) = UsbDevice::FromIdAsync(&device_id) {
//         if let Ok(usb_device) = operation.await {
//             if let Ok(desc) = usb_device.DeviceDescriptor() {
//                 return match (desc.ProductId(), desc.VendorId()) {
//                     (Ok(product_id), Ok(vendor_id)) => Some((product_id, vendor_id)),
//                     _ => {
//                         println!("Nope!!!");
//                         None
//                     }
//                 };
//             } else {
//                 println!("Tutaj4");
//             }
//         } else {
//             println!("Tutaj3");
//         }
//     } else {
//         println!("Tutaj2");
//     }

//     None
// }
