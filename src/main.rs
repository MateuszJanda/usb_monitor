use std::collections::HashSet;

use futures::executor::block_on;
use windows::core::*;
use windows::Devices::Enumeration::DeviceInformation;
use windows::Devices::Usb::UsbDevice;
use windows::Win32::{
    Foundation::*, Graphics::Gdi::ValidateRect, System::LibraryLoader::GetModuleHandleA,
    UI::WindowsAndMessaging::*,
};
use windows::Win32::System::SystemServices;

use std::mem::transmute;

fn main() -> Result<()> {
    let mut usb_monitor = UsbMonitor::new();

    usb_monitor.run()
}


struct UsbMonitor {
    handle: HWND,
    devices: HashSet<String>,
    // count: u32,
}

impl UsbMonitor {
    fn new() -> Self {
        UsbMonitor { handle: HWND(0), devices: HashSet::new() }
    }

    fn run(&mut self) -> Result<()> {
        unsafe {
            let instance = GetModuleHandleA(None)?;
            debug_assert!(instance.0 != 0);

            // Not predefined (by MS) class name, so must be registered by RegisterClassA
            let window_class = s!("usb_monitor_window");

            let wc = WNDCLASSA {
                hInstance: instance,
                lpszClassName: window_class,

                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(Self::wndproc),
                ..Default::default()
            };

            let atom = RegisterClassA(&wc);
            debug_assert!(atom != 0);

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


            // debug_assert!(handle.0 != 0);
            // debug_assert!(handle == self.handle);

            let mut message = MSG::default();

            // let future = get_all_usb_info();
            // let mmm = block_on(future);

            // println!("main {}", mmm.len());

            while GetMessageA(&mut message, HWND(0), 0, 0).into() {
                DispatchMessageA(&message);
            }

            Ok(())
        }
    }

    extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        unsafe {
            if message == WM_NCCREATE {
                println!("WM_NCCREATE");
                // let cs = lparam.0 as *const CREATESTRUCTA;
                // let this = (*cs).lpCreateParams as *mut Self;
                let cs: &CREATESTRUCTA = transmute(lparam);
                let this = cs.lpCreateParams as *mut Self;

                // if !this.is_null()
                // {
                    (*this).handle = window;

                // }
                // else
                // {
                //     println!("PROBLEM");
                // }


                SetWindowLongPtrA(window, GWLP_USERDATA, this as _);

                // SetWindowLongPtrA(window, GWLP_USERDATA, cs.lpCreateParams as _);


            } else {
                // println!("ELSE");
                let this = GetWindowLongPtrA(window, GWLP_USERDATA) as *mut Self;
                // (*this).handle = window;

                if !this.is_null() {
                    // println!("TUTAJ");
                    return (*this).message_handler(window, message, wparam, lparam);
                }
                else {
                    // println!("IS NULL");
                }
            }

            // println!("UPS something wrong");
            DefWindowProcA(window, message, wparam, lparam)
        }
    }

    fn message_handler(&mut self, window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        unsafe {
            // self.handle = window;
            match message as u32 {

                WM_CREATE => {

                    println!("WM_CREATE ");

                    // let cs = lparam.0 as *const CREATESTRUCTA;
                    // let this = (*cs).lpCreateParams as *mut Self;
                    // (*this).handle = window;

                    // let future = get_all_usb_info();
                    // self.devices = block_on(future);

                    println!("WM_CREATE {}", self.devices.len());

                    // (*this).count += 1;

                    LRESULT(0)
                },

                WM_PAINT => {
                    println!("WM_PAINT");
                    ValidateRect(self.handle, std::ptr::null());
                    LRESULT(0)
                },
                WM_DESTROY => {
                    println!("WM_DESTROY");
                    PostQuitMessage(0);
                    LRESULT(0)
                },
                WM_DEVICECHANGE => {



                    if wparam.0 as u32 == SystemServices::DBT_DEVICEARRIVAL  {
                            println!("DBT_DEVICEARRIVAL ");

                            let future = get_all_usb_info();
                            let nnnmmm = block_on(future);

                            // let cs = lparam.0 as *const CREATESTRUCTA;
                            // let this = (*cs).lpCreateParams as *mut Self;
                            // (*this).handle = window;


                            for device_id in nnnmmm {
                                if !self.devices.contains(&device_id) {

                                    // println!("New device {}: {}-{} [{}]", count, value.0, value.1, device_id);
                                    println!("New device: {}", device_id);

                                    // mmm.insert(device_id, value);
                                    self.devices.insert(device_id);
                                }
                            }

                            // count += 1;
                        }
                        // DBT_DEVICEREMOVECOMPLETE => {
                        //     println!("DBT_DEVICEREMOVECOMPLETE");
                        // }

                    LRESULT(0)
                },


                // _ => DefWindowProcA(self.handle, message, wparam, lparam),
                _ => DefWindowProcA(window, message, wparam, lparam),
            }
        }
    }

}

// https://docs.microsoft.com/en-us/windows-hardware/drivers/usbcon/how-to-connect-to-a-usb-device--uwp-app-#finding-the-devicethe-basic-way

async fn get_all_usb_info() -> HashSet<String>
{
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



                        }
                        else {
                            // println!("False");
                        }
                    }

                    else {
                        // println!("Not enabled");
                    }

                }
            }
        }
    }

    result
}

async fn usb_product_and_vendor(device_id:&HSTRING) -> Option<(u32, u32)>
{
    if let Ok(operation) = UsbDevice::FromIdAsync(&device_id) {
        if let Ok(usb_device) = operation.await {
            if let Ok(desc) = usb_device.DeviceDescriptor() {
                return match (desc.ProductId(), desc.VendorId()) {
                    (Ok(product_id), Ok(vendor_id)) => Some((product_id, vendor_id)),
                    _ => {
                        println!("Nope!!!");
                        None
                    },
                }
            }
            else
            {
                println!("Tutaj4");

            }
        }
        else {
            println!("Tutaj3");

        }
    }
    else
    {
        println!("Tutaj2");
    }

    None
}

