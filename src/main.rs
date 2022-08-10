use windows::core::*;
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

        while GetMessageA(&mut message, HWND(0), 0, 0).into() {
            DispatchMessageA(&message);
        }

        Ok(())
    }
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message as u32 {
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
                println!("WM_DEVICECHANGE");

                match wparam {
                    DBT_DEVICEARRIVAL => {
                        println!("DBT_DEVICEARRIVAL");
                    },
                    DBT_DEVICEREMOVECOMPLETE => {
                        println!("DBT_DEVICEREMOVECOMPLETE");
                    }
                 _ => ()
                }

                LRESULT(0)
            },
            _ => DefWindowProcA(window, message, wparam, lparam),
        }
    }
}
