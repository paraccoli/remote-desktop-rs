//! スクリーンショット取得モジュール
//!
//! システムからスクリーンショットをキャプチャするための機能を提供します。

use super::{CapturedImage, Monitor};
use crate::error::ServerError;

use image::{DynamicImage, RgbaImage, ImageBuffer, Rgba};
use thiserror::Error;
use std::time::Duration;

/// キャプチャエラー
#[derive(Error, Debug)]
pub enum CaptureError {
    /// キャプチャデバイスエラー
    #[error("キャプチャデバイスエラー: {0}")]
    DeviceError(String),
    
    /// キャプチャ処理エラー
    #[error("キャプチャ処理エラー: {0}")]
    ProcessError(String),
    
    /// 画像変換エラー
    #[error("画像変換エラー: {0}")]
    ImageConversionError(String),
    
    /// モニター設定エラー
    #[error("モニター設定エラー: {0}")]
    MonitorError(String),
    
    /// その他のエラー
    #[error("キャプチャエラー: {0}")]
    Other(String),
}

impl From<image::ImageError> for CaptureError {
    fn from(err: image::ImageError) -> Self {
        CaptureError::ImageConversionError(err.to_string())
    }
}

/// スクリーンキャプチャ
pub struct ScreenCapture {
    /// 最後に取得したスクリーンショット
    last_capture: Option<CapturedImage>,
    /// キャプチャ間の最小間隔（ミリ秒）
    min_interval: Duration,
    /// 最後のキャプチャ時刻
    last_capture_time: std::time::Instant,
    /// 使用するモニター
    monitor: Option<Monitor>,
}

impl ScreenCapture {
    /// 新しいスクリーンキャプチャを作成
    pub fn new() -> Self {
        Self {
            last_capture: None,
            min_interval: Duration::from_millis(50), // デフォルトは50ms（20fps相当）
            last_capture_time: std::time::Instant::now(),
            monitor: None,
        }
    }
    
    /// 最小キャプチャ間隔を設定
    pub fn set_min_interval(&mut self, interval: Duration) {
        self.min_interval = interval;
    }
    
    /// キャプチャするモニターを設定
    pub fn set_monitor(&mut self, monitor: Monitor) {
        self.monitor = Some(monitor);
    }
    
    /// すべてのモニターからキャプチャ
    pub fn capture_all_monitors(&mut self) -> Result<Vec<CapturedImage>, CaptureError> {
        let monitors = self.get_monitors()?;
        let mut images = Vec::with_capacity(monitors.len());
        
        for (index, monitor) in monitors.iter().enumerate() {
            match self.capture_monitor(index) {
                Ok(image) => images.push(image),
                Err(e) => {
                    log::warn!("モニター{}のキャプチャに失敗しました: {}", index, e);
                    // エラーがあっても続行し、他のモニターはキャプチャを試みる
                }
            }
        }
        
        if images.is_empty() {
            return Err(CaptureError::ProcessError("すべてのモニターのキャプチャに失敗しました".to_string()));
        }
        
        Ok(images)
    }
    
    /// 特定のモニターからキャプチャ
    pub fn capture_monitor(&mut self, monitor_index: usize) -> Result<CapturedImage, CaptureError> {
        // 最小間隔チェック
        let now = std::time::Instant::now();
        if now.duration_since(self.last_capture_time) < self.min_interval {
            // まだ間隔が短すぎる場合は最後のキャプチャを返す
            if let Some(last) = &self.last_capture {
                return Ok(last.clone());
            }
        }
        
        // キャプチャ処理は各プラットフォーム向けに実装
        let image = self.platform_capture_monitor(monitor_index)?;
        
        // 結果を保存
        self.last_capture = Some(image.clone());
        self.last_capture_time = now;
        
        Ok(image)
    }
    
    /// メインモニターからキャプチャ（デフォルトのディスプレイ）
    pub fn capture_main_monitor(&mut self) -> Result<CapturedImage, CaptureError> {
        self.capture_monitor(0)
    }
    
    /// キャプチャを有効なモニターから実行
    pub fn capture(&mut self) -> Result<CapturedImage, CaptureError> {
        if let Some(monitor) = &self.monitor {
            self.capture_monitor(monitor.index)
        } else {
            self.capture_main_monitor()
        }
    }
    
    /// 使用可能なモニターの一覧を取得
    pub fn get_monitors(&self) -> Result<Vec<Monitor>, CaptureError> {
        self.platform_get_monitors()
    }
    
    /// プラットフォーム固有のモニター一覧取得
    #[cfg(target_os = "windows")]
    fn platform_get_monitors(&self) -> Result<Vec<Monitor>, CaptureError> {
        use winapi::um::winuser::{EnumDisplayMonitors, GetMonitorInfoW, MONITORINFOEXW, MONITORINFO};
        use winapi::shared::windef::{HDC, HMONITOR, LPRECT, RECT};
        use winapi::um::wingdi::{CreateDCW, DeleteDC};
        use std::mem::{size_of, zeroed};
        use std::ptr::null_mut;
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;
        
        unsafe extern "system" fn enum_monitor_proc(
            hmonitor: HMONITOR,
            _: HDC,
            _: LPRECT,
            data: isize,
        ) -> i32 {
            let monitors = &mut *(data as *mut Vec<Monitor>);
            
            let mut monitor_info: MONITORINFOEXW = zeroed();
            monitor_info.cbSize = size_of::<MONITORINFOEXW>() as u32;
            
            if GetMonitorInfoW(hmonitor, &mut monitor_info as *mut MONITORINFOEXW as *mut MONITORINFO) != 0 {
                let rcMonitor = monitor_info.rcMonitor;
                let rcWork = monitor_info.rcWork;
                let is_primary = (monitor_info.dwFlags & 1) != 0;
                
                // デバイス名をOsStringに変換
                let name_len = monitor_info.szDevice.iter()
                    .position(|&c| c == 0)
                    .unwrap_or(monitor_info.szDevice.len());
                let device_name = OsString::from_wide(&monitor_info.szDevice[0..name_len]);
                
                let monitor = Monitor {
                    index: monitors.len(),
                    position: (rcMonitor.left, rcMonitor.top),
                    size: ((rcMonitor.right - rcMonitor.left) as u32, (rcMonitor.bottom - rcMonitor.top) as u32),
                    work_area: ((rcWork.right - rcWork.left) as u32, (rcWork.bottom - rcWork.top) as u32),
                    is_primary,
                    name: device_name.to_string_lossy().to_string(),
                    handle: hmonitor as usize,
                };
                
                monitors.push(monitor);
            }
            
            1 // 列挙を続ける
        }
        
        let mut monitors = Vec::new();
        unsafe {
            if EnumDisplayMonitors(
                null_mut(),
                null_mut(),
                Some(enum_monitor_proc),
                &mut monitors as *mut _ as isize,
            ) == 0 {
                return Err(CaptureError::MonitorError("モニター情報の取得に失敗しました".to_string()));
            }
        }
        
        // プライマリーモニターが先頭になるようにソート
        monitors.sort_by(|a, b| b.is_primary.cmp(&a.is_primary));
        
        // インデックスを更新
        for (i, monitor) in monitors.iter_mut().enumerate() {
            monitor.index = i;
        }
        
        Ok(monitors)
    }
    
    #[cfg(target_os = "linux")]
    fn platform_get_monitors(&self) -> Result<Vec<Monitor>, CaptureError> {
        // x11rbを使用したLinux実装
        #[cfg(feature = "x11-support")]
        {
            use x11rb::connection::Connection;
            use x11rb::protocol::xproto::{ConnectionExt, Screen, get_geometry};
            
            let (conn, screen_num) = x11rb::connect(None)
                .map_err(|e| CaptureError::MonitorError(format!("X11接続エラー: {}", e)))?;
            
            let screen = &conn.setup().roots[screen_num];
            
            // 単一モニターとして処理（複数モニターの詳細情報を取得するにはRandR拡張が必要）
            let monitor = Monitor {
                index: 0,
                position: (0, 0),
                size: (screen.width_in_pixels as u32, screen.height_in_pixels as u32),
                work_area: (screen.width_in_pixels as u32, screen.height_in_pixels as u32),
                is_primary: true,
                name: "Primary Display".to_string(),
                handle: screen.root as usize,
            };
            
            Ok(vec![monitor])
        }
        
        #[cfg(not(feature = "x11-support"))]
        {
            Err(CaptureError::MonitorError("X11サポートが有効になっていません".to_string()))
        }
    }
    
    #[cfg(target_os = "macos")]
    fn platform_get_monitors(&self) -> Result<Vec<Monitor>, CaptureError> {
        // macOS実装（CoreGraphicsを使用）
        #[cfg(feature = "macos-support")]
        {
            use core_foundation::base::TCFType;
            use core_foundation::array::{CFArrayRef, CFArray};
            use core_foundation::string::{CFString, CFStringRef};
            use core_graphics::display::{CGDisplay, CGDisplayBounds, CGMainDisplayID, CGDirectDisplayID};
            
            unsafe {
                // 全ディスプレイのIDを取得
                let mut display_count: u32 = 0;
                let mut result = core_graphics::display::CGGetOnlineDisplayList(0, std::ptr::null_mut(), &mut display_count);
                
                if result != 0 {
                    return Err(CaptureError::MonitorError("ディスプレイ一覧の取得に失敗しました".to_string()));
                }
                
                let mut display_ids: Vec<CGDirectDisplayID> = vec![0; display_count as usize];
                result = core_graphics::display::CGGetOnlineDisplayList(
                    display_count,
                    display_ids.as_mut_ptr(),
                    &mut display_count
                );
                
                if result != 0 {
                    return Err(CaptureError::MonitorError("ディスプレイ一覧の取得に失敗しました".to_string()));
                }
                
                let main_display_id = CGMainDisplayID();
                let mut monitors = Vec::new();
                
                for (i, &display_id) in display_ids.iter().enumerate() {
                    let is_primary = display_id == main_display_id;
                    
                    // ディスプレイの境界を取得
                    let bounds = CGDisplayBounds(display_id);
                    
                    let monitor = Monitor {
                        index: i,
                        position: (bounds.origin.x as i32, bounds.origin.y as i32),
                        size: (bounds.size.width as u32, bounds.size.height as u32),
                        work_area: (bounds.size.width as u32, bounds.size.height as u32), // macOSではwork_areaを簡単に取得できない
                        is_primary,
                        name: format!("Display {}", i),
                        handle: display_id as usize,
                    };
                    
                    monitors.push(monitor);
                }
                
                // プライマリーモニターが先頭になるようにソート
                monitors.sort_by(|a, b| b.is_primary.cmp(&a.is_primary));
                
                // インデックスを更新
                for (i, monitor) in monitors.iter_mut().enumerate() {
                    monitor.index = i;
                }
                
                Ok(monitors)
            }
        }
        
        #[cfg(not(feature = "macos-support"))]
        {
            Err(CaptureError::MonitorError("macOSサポートが有効になっていません".to_string()))
        }
    }
    
    // フォールバック実装
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    fn platform_get_monitors(&self) -> Result<Vec<Monitor>, CaptureError> {
        // 単一ディスプレイをダミーデータとして返す
        let monitor = Monitor {
            index: 0,
            position: (0, 0),
            size: (1920, 1080), // デフォルトサイズ
            work_area: (1920, 1080),
            is_primary: true,
            name: "Default Display".to_string(),
            handle: 0,
        };
        
        Ok(vec![monitor])
    }
    
    /// プラットフォーム固有のキャプチャ実装
    #[cfg(target_os = "windows")]
    fn platform_capture_monitor(&self, monitor_index: usize) -> Result<CapturedImage, CaptureError> {
        use winapi::um::winuser::{GetDesktopWindow, GetDC, GetWindowDC};
        use winapi::um::wingdi::{BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, SelectObject, SRCCOPY, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS};
        use winapi::shared::windef::{HDC, HBITMAP, RECT};
        use winapi::shared::minwindef::{DWORD, WORD};
        use std::mem::{size_of, zeroed};
        use std::ptr::null_mut;
        
        // モニター情報を取得
        let monitors = self.get_monitors()?;
        let monitor = monitors.get(monitor_index).ok_or_else(|| {
            CaptureError::MonitorError(format!("モニターインデックス{}は範囲外です", monitor_index))
        })?;
        
        unsafe {
            // デスクトップウィンドウのDCを取得
            let hwnd = GetDesktopWindow();
            let h_screen_dc = GetDC(hwnd);
            if h_screen_dc.is_null() {
                return Err(CaptureError::DeviceError("デスクトップDCの取得に失敗しました".to_string()));
            }
            
            // 互換DCを作成
            let h_memory_dc = CreateCompatibleDC(h_screen_dc);
            if h_memory_dc.is_null() {
                DeleteDC(h_screen_dc);
                return Err(CaptureError::DeviceError("互換DCの作成に失敗しました".to_string()));
            }
            
            // モニターのサイズ情報
            let (width, height) = monitor.size;
            let (x, y) = monitor.position;
            
            // 互換ビットマップを作成
            let h_bitmap = CreateCompatibleBitmap(h_screen_dc, width as i32, height as i32);
            if h_bitmap.is_null() {
                DeleteDC(h_memory_dc);
                DeleteDC(h_screen_dc);
                return Err(CaptureError::DeviceError("互換ビットマップの作成に失敗しました".to_string()));
            }
            
            // ビットマップを選択
            let h_old_bitmap = SelectObject(h_memory_dc, h_bitmap as _);
            
            // スクリーンをキャプチャ
            let result = BitBlt(
                h_memory_dc,
                0, 0, width as i32, height as i32,
                h_screen_dc,
                x, y,
                SRCCOPY,
            );
            
            if result == 0 {
                SelectObject(h_memory_dc, h_old_bitmap);
                DeleteObject(h_bitmap as _);
                DeleteDC(h_memory_dc);
                DeleteDC(h_screen_dc);
                return Err(CaptureError::ProcessError("ビットブロック転送に失敗しました".to_string()));
            }
            
            // ビットマップデータを格納するバッファ
            let bytes_per_pixel = 4; // RGBA
            let stride = ((width as usize * bytes_per_pixel + 3) / 4) * 4;
            let buffer_size = stride * height as usize;
            let mut buffer = vec![0u8; buffer_size];
            
            // BITMAPINFO構造体を設定
            let mut bitmap_info: BITMAPINFO = zeroed();
            bitmap_info.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
            bitmap_info.bmiHeader.biWidth = width as i32;
            bitmap_info.bmiHeader.biHeight = -(height as i32); // 上から下へ
            bitmap_info.bmiHeader.biPlanes = 1;
            bitmap_info.bmiHeader.biBitCount = 32;
            bitmap_info.bmiHeader.biCompression = BI_RGB;
            
            // ビットマップデータを取得
            let scan_lines = GetDIBits(
                h_memory_dc,
                h_bitmap,
                0,
                height,
                buffer.as_mut_ptr() as _,
                &mut bitmap_info,
                DIB_RGB_COLORS,
            );
            
            // クリーンアップ
            SelectObject(h_memory_dc, h_old_bitmap);
            DeleteObject(h_bitmap as _);
            DeleteDC(h_memory_dc);
            DeleteDC(h_screen_dc);
            
            if scan_lines == 0 || scan_lines != height {
                return Err(CaptureError::ProcessError("ビットマップデータの取得に失敗しました".to_string()));
            }
            
            // BGRA -> RGBA に変換
            for chunk in buffer.chunks_exact_mut(4) {
                let b = chunk[0];
                chunk[0] = chunk[2]; // R <- B
                chunk[2] = b;        // B <- R
            }
            
            // ImageBufferを作成
            let image_buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, buffer)
                .ok_or_else(|| CaptureError::ImageConversionError("画像バッファの作成に失敗しました".to_string()))?;
            
            // DynamicImageに変換
            let dynamic_image = DynamicImage::ImageRgba8(image_buffer);
            
            Ok(CapturedImage::new(dynamic_image, monitor_index))
        }
    }
    
    #[cfg(target_os = "linux")]
    fn platform_capture_monitor(&self, monitor_index: usize) -> Result<CapturedImage, CaptureError> {
        #[cfg(feature = "x11-support")]
        {
            use x11rb::connection::Connection;
            use x11rb::protocol::xproto::{ConnectionExt, get_geometry, get_image, ImageFormat as X11ImageFormat};
            
            // モニター情報を取得
            let monitors = self.get_monitors()?;
            let monitor = monitors.get(monitor_index).ok_or_else(|| {
                CaptureError::MonitorError(format!("モニターインデックス{}は範囲外です", monitor_index))
            })?;
            
            let (width, height) = monitor.size;
            let (x, y) = monitor.position;
            
            let (conn, _) = x11rb::connect(None)
                .map_err(|e| CaptureError::DeviceError(format!("X11接続エラー: {}", e)))?;
            
            let root = conn.setup().roots[0].root;
            
            // 画像を取得
            let image_reply = conn.get_image(
                X11ImageFormat::Z_PIXMAP.into(),
                root,
                x as i16, y as i16,
                width, height,
                !0, // プレーンマスク（すべて）
            )
            .map_err(|e| CaptureError::ProcessError(format!("X11 get_image失敗: {}", e)))?
            .reply()
            .map_err(|e| CaptureError::ProcessError(format!("X11 reply失敗: {}", e)))?;
            
            let data = image_reply.data;
            
            // BGRAからRGBAに変換
            let mut rgba_data = Vec::with_capacity(data.len());
            for i in (0..data.len()).step_by(4) {
                if i + 3 < data.len() {
                    rgba_data.push(data[i + 2]); // R
                    rgba_data.push(data[i + 1]); // G
                    rgba_data.push(data[i]);     // B
                    rgba_data.push(data[i + 3]); // A
                }
            }
            
            // ImageBufferを作成
            let image_buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, rgba_data)
                .ok_or_else(|| CaptureError::ImageConversionError("画像バッファの作成に失敗しました".to_string()))?;
            
            // DynamicImageに変換
            let dynamic_image = DynamicImage::ImageRgba8(image_buffer);
            
            Ok(CapturedImage::new(dynamic_image, monitor_index))
        }
        
        #[cfg(not(feature = "x11-support"))]
        {
            Err(CaptureError::MonitorError("X11サポートが有効になっていません".to_string()))
        }
    }
    
    #[cfg(target_os = "macos")]
    fn platform_capture_monitor(&self, monitor_index: usize) -> Result<CapturedImage, CaptureError> {
        #[cfg(feature = "macos-support")]
        {
            use core_graphics::display::{CGDisplayCreateImage, CGDirectDisplayID};
            use core_graphics::image::{CGImageRef};
            use core_foundation::base::TCFType;
            
            // モニター情報を取得
            let monitors = self.get_monitors()?;
            let monitor = monitors.get(monitor_index).ok_or_else(|| {
                CaptureError::MonitorError(format!("モニターインデックス{}は範囲外です", monitor_index))
            })?;
            
            let display_id = monitor.handle as CGDirectDisplayID;
            
            unsafe {
                // ディスプレイのキャプチャ
                let cg_image = CGDisplayCreateImage(display_id);
                if cg_image.is_null() {
                    return Err(CaptureError::ProcessError("CGDisplayCreateImage失敗".to_string()));
                }
                
                // CGImageRefからBGRAバイト配列に変換
                let width = core_graphics::image::CGImageGetWidth(cg_image);
                let height = core_graphics::image::CGImageGetHeight(cg_image);
                let bytes_per_row = core_graphics::image::CGImageGetBytesPerRow(cg_image);
                let data_provider = core_graphics::image::CGImageGetDataProvider(cg_image);
                let data = core_graphics::data_provider::CGDataProviderCopyData(data_provider);
                let bytes = core_foundation::data::CFDataGetBytePtr(data);
                let length = core_foundation::data::CFDataGetLength(data) as usize;
                
                // BGRAデータをRGBAに変換
                let mut rgba_data = Vec::with_capacity(width as usize * height as usize * 4);
                for y in 0..height {
                    for x in 0..width {
                        let offset = (y * bytes_per_row + x * 4) as isize;
                        rgba_data.push(*bytes.offset(offset + 2)); // R <- B
                        rgba_data.push(*bytes.offset(offset + 1)); // G
                        rgba_data.push(*bytes.offset(offset));     // B <- R
                        rgba_data.push(*bytes.offset(offset + 3)); // A
                    }
                }
                
                // リソース解放
                core_foundation::base::CFRelease(data as _);
                core_foundation::base::CFRelease(cg_image as _);
                
                // ImageBufferを作成
                let image_buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, rgba_data)
                    .ok_or_else(|| CaptureError::ImageConversionError("画像バッファの作成に失敗しました".to_string()))?;
                
                // DynamicImageに変換
                let dynamic_image = DynamicImage::ImageRgba8(image_buffer);
                
                Ok(CapturedImage::new(dynamic_image, monitor_index))
            }
        }
        
        #[cfg(not(feature = "macos-support"))]
        {
            Err(CaptureError::MonitorError("macOSサポートが有効になっていません".to_string()))
        }
    }
    
    // プラットフォーム非依存のフォールバック実装（ダミーデータ）
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    fn platform_capture_monitor(&self, monitor_index: usize) -> Result<CapturedImage, CaptureError> {
        // ダミーの黒い画像を生成（開発・テスト用）
        let width = 1280;
        let height = 720;
        
        let image_buffer = RgbaImage::from_fn(width, height, |x, y| {
            let r = x as u8;
            let g = y as u8;
            let b = ((x + y) / 2) as u8;
            Rgba([r, g, b, 255])
        });
        
        let dynamic_image = DynamicImage::ImageRgba8(image_buffer);
        
        Ok(CapturedImage::new(dynamic_image, monitor_index))
    }
}

impl Default for ScreenCapture {
    fn default() -> Self {
        Self::new()
    }
}