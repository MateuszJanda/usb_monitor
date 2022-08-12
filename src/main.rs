use std::collections::BTreeMap;

use futures::executor::block_on;
use windows::core::*;
use windows::Devices::Enumeration::DeviceInformation;
use windows::Devices::Usb::UsbDevice;
use windows::Win32::{
    Foundation::*, Graphics::Gdi::ValidateRect, System::LibraryLoader::GetModuleHandleA,
    UI::WindowsAndMessaging::*,
};



fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleA(None)?;
        debug_assert!(instance.0 != 0);

        // Not predefined (by MS) class name, so must be registered by RegisterClassA
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

        CreateWindowExA(
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
            std::ptr::null(),
        );

        let mut message = MSG::default();

        let future = get_all_usb_info();
        let mmm = block_on(future);

        // println!("main {}", mmm.len());

        while GetMessageA(&mut message, HWND(0), 0, 0).into() {
            DispatchMessageA(&message);
        }

        Ok(())
    }
}

// https://docs.microsoft.com/en-us/windows-hardware/drivers/usbcon/how-to-connect-to-a-usb-device--uwp-app-#finding-the-devicethe-basic-way

async fn get_all_usb_info() -> BTreeMap<String, (u32, u32)>
{
    let mut result = BTreeMap::new();


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

                        result.insert(device_id.to_string(), (0,0));
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

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        let mut mmm = BTreeMap::new();
        let mut count = 0;

        match message as u32 {
            WM_CREATE => {
                let future = get_all_usb_info();
                mmm = block_on(future);

                println!("WM_CREATE {}", mmm.len());

                LRESULT(0)
            },

            WM_PAINT => {
                println!("WM_PAINT");
                ValidateRect(window, std::ptr::null());
                LRESULT(0)
            },
            WM_DESTROY => {
                println!("WM_DESTROY");
                PostQuitMessage(0);
                LRESULT(0)
            },
            WM_DEVICECHANGE => {

                match wparam {
                    DBT_DEVICEARRIVAL => {
                        println!("DBT_DEVICEARRIVAL");

                        let future = get_all_usb_info();
                        let nnnmmm = block_on(future);

                        for (device_id, value) in nnnmmm {
                            if !mmm.contains_key(&device_id) {

                                println!("New device {}: {}-{} [{}]", count, value.0, value.1, device_id);

                                mmm.insert(device_id, value);
                            }
                        }

                        count += 1;
                    },
                    // DBT_DEVICEREMOVECOMPLETE => {
                    //     println!("DBT_DEVICEREMOVECOMPLETE");
                    // }
                 _ => ()
                }

                LRESULT(0)
            },
            _ => DefWindowProcA(window, message, wparam, lparam),
        }
    }
}
