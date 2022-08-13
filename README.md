Toy program to monitor newly plugged USB devices to Windows PC. it use [windows-rs](https://github.com/microsoft/windows-rs)
to call Win32 and UWP (Universal Windows Platform) API.

```
WM_NCCREATE. Saving pointer to UsbMonitor object.
WM_CREATE. Archived USB info: 0
DBT_DEVICEARRIVAL. New device plugged.
New USB info: \\?\USB#VID_1004&PID_61C5#D918C66F1D7D311B28F021B17E#{9c9782c3-6fd4-4244-81d0-690a029c6369}
New USB info: \\?\USB#VID_04E8&PID_3255#FE7D76BC7BB07B2077DC600759#{db123e99-2dcf-4fa2-8755-5ac3f5b17720}
Archived USB info: 2
```