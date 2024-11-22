use std::{ffi::c_void, io::Cursor, sync::OnceLock};

use async_trait::async_trait;
use base64::{prelude::BASE64_STANDARD, Engine as _};
use image::{ImageBuffer, ImageFormat, Rgba};
use serde::{Deserialize, Serialize};
use windows::{
  core::{Error, Interface, HSTRING, PWSTR},
  Win32::{
    Foundation::{BOOL, E_FAIL, HWND, LPARAM, SIZE, WPARAM},
    Graphics::Gdi::{
      DeleteObject, GetDC, GetDIBits, ReleaseDC, BITMAPINFO,
      BITMAPINFOHEADER, DIB_RGB_COLORS, HBITMAP, RGBQUAD,
    },
    Storage::Packaging::Appx::GetApplicationUserModelId,
    System::{
      Com::{CoInitializeEx, COINIT_APARTMENTTHREADED},
      Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION},
    },
    UI::{
      Accessibility::{SetWinEventHook, HWINEVENTHOOK},
      Shell::{
        IShellItem, IShellItemImageFactory, SHCreateItemFromParsingName,
        SIIGBF_BIGGERSIZEOK,
      },
      WindowsAndMessaging::{
        DestroyIcon, DispatchMessageW, EnumChildWindows, GetClassLongPtrW,
        GetForegroundWindow, GetIconInfo, GetMessageW,
        GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
        IsWindow, SendMessageW, TranslateMessage, EVENT_OBJECT_CREATE,
        EVENT_OBJECT_NAMECHANGE, EVENT_SYSTEM_FOREGROUND,
        EVENT_SYSTEM_SWITCHSTART, GCLP_HICONSM, HICON, ICONINFO, ICON_BIG,
        ICON_SMALL, MSG, WINEVENT_OUTOFCONTEXT, WM_GETICON,
      },
    },
  },
};

use crate::providers::{
  CommonProviderState, Provider, ProviderEmitter, RuntimeType,
};

static PROVIDER_TX: OnceLock<ProviderEmitter> = OnceLock::new();

// This doesn't always work..
const ICON_SIZE: i32 = 32;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FocusedWindowProviderConfig {}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FocusedWindowOutput {
  pub title: String,
  pub icon: String,
}

pub struct FocusedWindowProvider {
  common: CommonProviderState,
}

impl FocusedWindowProvider {
  pub fn new(
    _config: FocusedWindowProviderConfig,
    common: CommonProviderState,
  ) -> FocusedWindowProvider {
    FocusedWindowProvider { common }
  }

  unsafe fn get_foreground_window_icon(
    hwnd: HWND,
  ) -> Result<String, Error> {
    // Try standard window icon first
    if let Some(b64) = Self::extract_standard_icon(hwnd) {
      return Ok(b64);
    }

    // Try UWP icon
    if let Some(aumid) = Self::find_uwp_app_id(hwnd) {
      if let Some(b64) = Self::extract_uwp_icon(&aumid) {
        return Ok(b64);
      }
    }

    Err(Error::new(E_FAIL, "Failed to extract icon"))
  }

  unsafe fn extract_standard_icon(hwnd: HWND) -> Option<String> {
    let hicon =
      SendMessageW(hwnd, WM_GETICON, WPARAM(ICON_BIG as usize), LPARAM(0));
    let hicon = if hicon.0 != 0 {
      HICON(hicon.0 as *mut _)
    } else {
      let hicon = SendMessageW(
        hwnd,
        WM_GETICON,
        WPARAM(ICON_SMALL as usize),
        LPARAM(0),
      );
      if hicon.0 != 0 {
        HICON(hicon.0 as *mut _)
      } else {
        let hicon = GetClassLongPtrW(hwnd, GCLP_HICONSM);
        if hicon != 0 {
          HICON(hicon as *mut _)
        } else {
          return None;
        }
      }
    };

    let mut icon_info = ICONINFO::default();
    if GetIconInfo(hicon, &mut icon_info).is_ok() {
      let bitmap = icon_info.hbmColor;
      // TODO (check the docs more)
      let width = icon_info.xHotspot * 2;
      let height = icon_info.yHotspot * 2;
      let b64 =
        Self::bitmap_to_base64(bitmap, width as u32, height as u32);
      let _ = DeleteObject(icon_info.hbmColor);
      let _ = DeleteObject(icon_info.hbmMask);
      let _ = DestroyIcon(hicon);
      b64
    } else {
      let _ = DestroyIcon(hicon);
      None
    }
  }

  unsafe fn extract_uwp_icon(aumid: &str) -> Option<String> {
    let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
    let shell_path = format!("shell:AppsFolder\\{}", aumid);

    let shell_item: IShellItem =
      SHCreateItemFromParsingName(&HSTRING::from(shell_path), None)
        .ok()?;
    let image_factory: IShellItemImageFactory = shell_item.cast().ok()?;

    let size = SIZE {
      cx: ICON_SIZE,
      cy: ICON_SIZE,
    };

    let hbitmap =
      image_factory.GetImage(size, SIIGBF_BIGGERSIZEOK).ok()?;
    let hbmp = HBITMAP(hbitmap.0);

    if !hbmp.is_invalid() {
      let result =
        Self::bitmap_to_base64(hbmp, ICON_SIZE as u32, ICON_SIZE as u32);
      let _ = DeleteObject(hbmp);
      result
    } else {
      let _ = DeleteObject(hbmp);
      None
    }
  }

  unsafe fn find_uwp_app_id(hwnd: HWND) -> Option<String> {
    let mut process_id = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut process_id));

    // Try the main window's process
    if let Ok(handle) =
      OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id)
    {
      let mut len = 0u32;
      let _ = GetApplicationUserModelId(handle, &mut len, PWSTR::null());
      if len > 0 {
        let mut buffer = vec![0u16; len as usize];
        if GetApplicationUserModelId(
          handle,
          &mut len,
          PWSTR(buffer.as_mut_ptr()),
        )
        .is_ok()
        {
          return Some(String::from_utf16_lossy(
            &buffer[..len as usize - 1],
          ));
        }
      }
    }

    // Try child windows
    let mut uwp_app_id = None;
    let _ = EnumChildWindows(
      hwnd,
      Some(enum_child_proc),
      LPARAM(&mut uwp_app_id as *mut _ as isize),
    );
    uwp_app_id
  }

  unsafe fn get_foreground_window_title(hwnd: HWND) -> Option<String> {
    if hwnd.0 == std::ptr::null_mut() {
      return None;
    }

    let length = GetWindowTextLengthW(hwnd);
    if length == 0 {
      return None;
    }

    let mut buffer: Vec<u16> = vec![0; (length + 1) as usize];
    GetWindowTextW(hwnd, &mut buffer);
    let title = String::from_utf16_lossy(&buffer[..length as usize]);
    Some(title)
  }

  unsafe fn bitmap_to_base64(
    bitmap: HBITMAP,
    width: u32,
    height: u32,
  ) -> Option<String> {
    // Get bitmap data
    let mut bmi = BITMAPINFO {
      bmiHeader: BITMAPINFOHEADER {
        biSize: size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: width as i32,
        biHeight: -(height as i32), // Negative height for top-down DIB
        biPlanes: 1,
        biBitCount: 32,
        biCompression: 0,
        biSizeImage: 0,
        biXPelsPerMeter: 0,
        biYPelsPerMeter: 0,
        biClrUsed: 0,
        biClrImportant: 0,
      },
      bmiColors: [RGBQUAD::default()],
    };

    let mut pixels: Vec<u8> = vec![0; (width * height * 4) as usize];
    let hdc = GetDC(None);

    let result = GetDIBits(
      hdc,
      bitmap,
      0,
      height,
      Some(pixels.as_mut_ptr() as *mut c_void),
      &mut bmi,
      DIB_RGB_COLORS,
    );

    ReleaseDC(None, hdc);

    if result == 0 {
      return None;
    }

    // Convert BGR to RGB
    for pixel in pixels.chunks_exact_mut(4) {
      // Swap B and R channels, leaving A and G unchanged
      pixel.swap(0, 2);
    }

    // Convert to PNG and then base64
    let mut png_data = Vec::new();
    let mut cursor = Cursor::new(&mut png_data);

    let image_buffer =
      ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, pixels)?;
    image_buffer.write_to(&mut cursor, ImageFormat::Png).ok()?;

    Some(BASE64_STANDARD.encode(&png_data))
  }

  unsafe fn emit_window_info(hwnd: HWND, emitter: &ProviderEmitter) {
    println!("Emitting window info");
    if !IsWindow(hwnd).as_bool() {
      return;
    }
    let output = Self::focused_window_output(hwnd);
    emitter.emit_output(output);
  }

  unsafe fn focused_window_output(
    hwnd: HWND,
  ) -> anyhow::Result<FocusedWindowOutput> {
    if let Some(title) = Self::get_foreground_window_title(hwnd) {
      if title.trim().is_empty() {
        return Err(anyhow::Error::msg("Empty title"));
      }

      if let Ok(icon) = Self::get_foreground_window_icon(hwnd) {
        Ok(FocusedWindowOutput { title, icon })
      } else {
        Err(anyhow::Error::msg("Failed to extract icon"))
      }
    } else {
      Err(anyhow::Error::msg("Failed to get window title"))
    }
  }

  fn create_focused_window_hook(&self) -> anyhow::Result<()> {
    PROVIDER_TX
      .set(self.common.emitter.clone())
      .expect("Error setting provider tx in focused window provider");

    unsafe {
      let focus_hook = SetWinEventHook(
        EVENT_SYSTEM_FOREGROUND,
        EVENT_SYSTEM_FOREGROUND,
        None,
        Some(win_event_proc),
        0,
        0,
        WINEVENT_OUTOFCONTEXT,
      );

      let title_hook = SetWinEventHook(
        EVENT_OBJECT_NAMECHANGE,
        EVENT_OBJECT_NAMECHANGE,
        None,
        Some(win_event_proc),
        0,
        0,
        WINEVENT_OUTOFCONTEXT,
      );

      let create_hook = SetWinEventHook(
        EVENT_OBJECT_CREATE,
        EVENT_OBJECT_CREATE,
        None,
        Some(win_event_proc),
        0,
        0,
        WINEVENT_OUTOFCONTEXT,
      );

      let switch_hook = SetWinEventHook(
        EVENT_SYSTEM_SWITCHSTART,
        EVENT_SYSTEM_SWITCHSTART,
        None,
        Some(win_event_proc),
        0,
        0,
        WINEVENT_OUTOFCONTEXT,
      );

      if focus_hook.is_invalid() || title_hook.is_invalid() {
        return Err(anyhow::Error::msg("Failed to set event hooks"));
      }

      // Initialize with current foreground window
      if let Some(current_hwnd) = Some(GetForegroundWindow()) {
        Self::emit_window_info(current_hwnd, &self.common.emitter);
      }

      let mut msg = MSG::default();
      while GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() {
        let _ = TranslateMessage(&msg);
        let _ = DispatchMessageW(&msg);
      }

      Ok(())
    }
  }
}

unsafe extern "system" fn win_event_proc(
  hook: HWINEVENTHOOK,
  event: u32,
  hwnd: HWND,
  id_object: i32,
  id_child: i32,
  thread_id: u32,
  time: u32,
) {
  if !IsWindow(hwnd).as_bool() {
    return;
  }
  if let Some(emitter) = PROVIDER_TX.get() {
    match event {
      EVENT_SYSTEM_FOREGROUND => {
        FocusedWindowProvider::emit_window_info(hwnd, emitter);
      }
      EVENT_OBJECT_CREATE => {
        let foreground = GetForegroundWindow();
        if hwnd == foreground {
          FocusedWindowProvider::emit_window_info(hwnd, emitter);
        }
      }
      EVENT_OBJECT_NAMECHANGE => {
        if id_object == 0 && hwnd == GetForegroundWindow() {
          FocusedWindowProvider::emit_window_info(hwnd, emitter);
        }
      }
      EVENT_SYSTEM_SWITCHSTART => {
        let hwnd = GetForegroundWindow();
        if !hwnd.is_invalid() {
          FocusedWindowProvider::emit_window_info(hwnd, emitter);
        }
      }
      _ => {}
    }
  }
}

unsafe extern "system" fn enum_child_proc(
  child_hwnd: HWND,
  lparam: LPARAM,
) -> BOOL {
  let result = lparam.0 as *mut Option<String>;
  let mut child_process_id = 0;
  GetWindowThreadProcessId(child_hwnd, Some(&mut child_process_id));

  if let Ok(handle) =
    OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, child_process_id)
  {
    let mut len = 0u32;
    let _ = GetApplicationUserModelId(handle, &mut len, PWSTR::null());
    if len > 0 {
      let mut buffer = vec![0u16; len as usize];
      if GetApplicationUserModelId(
        handle,
        &mut len,
        PWSTR(buffer.as_mut_ptr()),
      )
      .is_ok()
      {
        unsafe {
          *result =
            Some(String::from_utf16_lossy(&buffer[..len as usize - 1]));
        }
        return BOOL(0);
      }
    }
  }
  BOOL(1)
}

#[async_trait]
impl Provider for FocusedWindowProvider {
  fn runtime_type(&self) -> RuntimeType {
    RuntimeType::Sync
  }

  fn start_sync(&mut self) {
    if let Err(err) = self.create_focused_window_hook() {
      self
        .common
        .emitter
        .emit_output::<FocusedWindowOutput>(Err(err));
    }
  }
}
