//! 画像差分計算モジュール
//!
//! 連続する画面キャプチャ間の差分を効率的に計算し、
//! 変更のあった領域のみを送信するための機能を提供します。

use super::CapturedImage;
use image::{DynamicImage, GenericImageView, Rgba};
use std::collections::HashSet;

/// 矩形領域
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rectangle {
    /// X座標
    pub x: u32,
    /// Y座標
    pub y: u32,
    /// 幅
    pub width: u32,
    /// 高さ
    pub height: u32,
}

impl Rectangle {
    /// 新しい矩形を作成
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }
    
    /// 矩形の面積を取得
    pub fn area(&self) -> u32 {
        self.width * self.height
    }
    
    /// 矩形が別の矩形と重なっているか確認
    pub fn overlaps(&self, other: &Rectangle) -> bool {
        self.x < other.x + other.width &&
        self.x + self.width > other.x &&
        self.y < other.y + other.height &&
        self.y + self.height > other.y
    }
    
    /// 2つの矩形を統合した新しい矩形を作成
    pub fn merge(&self, other: &Rectangle) -> Self {
        let min_x = self.x.min(other.x);
        let min_y = self.y.min(other.y);
        let max_x = (self.x + self.width).max(other.x + other.width);
        let max_y = (self.y + self.height).max(other.y + other.height);
        
        Rectangle::new(
            min_x,
            min_y,
            max_x - min_x,
            max_y - min_y,
        )
    }
}

/// 差分結果
#[derive(Debug, Clone)]
pub struct DiffResult {
    /// 変更された領域
    pub changed_regions: Vec<Rectangle>,
    /// 変更されたピクセル数
    pub changed_pixels: u32,
    /// 合計ピクセル数
    pub total_pixels: u32,
    /// 変更率（0.0～1.0）
    pub change_ratio: f32,
}

/// 差分計算の設定
#[derive(Debug, Clone, Copy)]
pub struct DiffConfig {
    /// ブロックサイズ（分割する正方形のサイズ）
    pub block_size: u32,
    /// しきい値（0～255）- この値以上の差異があるピクセルを「変更あり」と判定
    pub threshold: u8,
    /// 変化したとみなすブロック内のピクセル比率
    pub change_ratio_threshold: f32,
    /// 隣接する差分領域をマージする
    pub merge_adjacent: bool,
    /// 最小差分サイズ（これより小さい差分領域は無視）
    pub min_diff_size: u32,
}

impl Default for DiffConfig {
    fn default() -> Self {
        Self {
            block_size: 32,
            threshold: 15,
            change_ratio_threshold: 0.05, // ブロック内の5%以上のピクセルが変化した場合に差分とみなす
            merge_adjacent: true,
            min_diff_size: 8, // 8x8より小さい差分領域は無視
        }
    }
}

/// 差分計算機
pub struct DiffCalculator {
    /// 設定
    config: DiffConfig,
    /// 前回の画像
    previous: Option<DynamicImage>,
}

impl DiffCalculator {
    /// 新しい差分計算機を作成
    pub fn new(config: DiffConfig) -> Self {
        Self {
            config,
            previous: None,
        }
    }
    
    /// 設定を更新
    pub fn set_config(&mut self, config: DiffConfig) {
        self.config = config;
    }
    
    /// 設定を取得
    pub fn config(&self) -> &DiffConfig {
        &self.config
    }
    
    /// 前回の画像をクリア
    pub fn clear_previous(&mut self) {
        self.previous = None;
    }
    
    /// 前回の画像を設定
    pub fn set_previous(&mut self, image: &CapturedImage) {
        self.previous = Some(image.image.clone());
    }
    
    /// 差分を計算
    pub fn calculate(&mut self, current: &CapturedImage) -> DiffResult {
        let current_img = &current.image;
        
        // 前回の画像がなければ、画面全体を差分と判定
        if self.previous.is_none() {
            let width = current_img.width();
            let height = current_img.height();
            let changed_region = Rectangle::new(0, 0, width, height);
            let total_pixels = width * height;
            
            // 現在の画像を保存
            self.previous = Some(current_img.clone());
            
            return DiffResult {
                changed_regions: vec![changed_region],
                changed_pixels: total_pixels,
                total_pixels,
                change_ratio: 1.0,
            };
        }
        
        let prev_img = self.previous.as_ref().unwrap();
        
        // 画像サイズが変わっていれば、画面全体を差分と判定
        if prev_img.width() != current_img.width() || prev_img.height() != current_img.height() {
            let width = current_img.width();
            let height = current_img.height();
            let changed_region = Rectangle::new(0, 0, width, height);
            let total_pixels = width * height;
            
            // 現在の画像を保存
            self.previous = Some(current_img.clone());
            
            return DiffResult {
                changed_regions: vec![changed_region],
                changed_pixels: total_pixels,
                total_pixels,
                change_ratio: 1.0,
            };
        }
        
        let width = current_img.width();
        let height = current_img.height();
        let block_size = self.config.block_size;
        let threshold = self.config.threshold;
        let change_threshold = self.config.change_ratio_threshold;
        let total_pixels = width * height;
        
        // ブロック単位で差分を計算
        let mut changed_blocks = HashSet::new();
        let mut changed_pixels_count = 0;
        
        for by in (0..height).step_by(block_size as usize) {
            for bx in (0..width).step_by(block_size as usize) {
                let block_width = block_size.min(width - bx);
                let block_height = block_size.min(height - by);
                let block_pixels = block_width * block_height;
                let mut block_changed_pixels = 0;
                
                // ブロック内の各ピクセルを比較
                for y in by..by + block_height {
                    for x in bx..bx + block_width {
                        let current_pixel = current_img.get_pixel(x, y);
                        let prev_pixel = prev_img.get_pixel(x, y);
                        
                        // ピクセル差を計算
                        let diff = pixel_diff(current_pixel, prev_pixel);
                        
                        if diff > threshold {
                            block_changed_pixels += 1;
                        }
                    }
                }
                
                // ブロック内の変更率を計算
                let block_change_ratio = block_changed_pixels as f32 / block_pixels as f32;
                
                // 閾値を超えたブロックを「変更あり」として記録
                if block_change_ratio >= change_threshold {
                    changed_blocks.insert(Rectangle::new(bx, by, block_width, block_height));
                    changed_pixels_count += block_changed_pixels;
                }
            }
        }
        
        // 隣接するブロックをマージ
        let mut changed_regions = if self.config.merge_adjacent {
            merge_adjacent_regions(changed_blocks.into_iter().collect())
        } else {
            changed_blocks.into_iter().collect()
        };
        
        // 最小サイズ以下の領域を除外
        if self.config.min_diff_size > 0 {
            changed_regions.retain(|r| r.width >= self.config.min_diff_size && r.height >= self.config.min_diff_size);
        }
        
        // 変更率を計算
        let change_ratio = changed_pixels_count as f32 / total_pixels as f32;
        
        // 現在の画像を次回比較用に保存
        self.previous = Some(current_img.clone());
        
        DiffResult {
            changed_regions,
            changed_pixels: changed_pixels_count,
            total_pixels,
            change_ratio,
        }
    }
}

impl Default for DiffCalculator {
    fn default() -> Self {
        Self::new(DiffConfig::default())
    }
}

/// ピクセル間の色差を計算
fn pixel_diff(p1: Rgba<u8>, p2: Rgba<u8>) -> u8 {
    let r_diff = (p1[0] as i16 - p2[0] as i16).abs() as u32;
    let g_diff = (p1[1] as i16 - p2[1] as i16).abs() as u32;
    let b_diff = (p1[2] as i16 - p2[2] as i16).abs() as u32;
    
    // 色差の平均値
    ((r_diff + g_diff + b_diff) / 3) as u8
}

/// 隣接する領域をマージする
fn merge_adjacent_regions(regions: Vec<Rectangle>) -> Vec<Rectangle> {
    if regions.is_empty() {
        return regions;
    }
    
    let mut result = Vec::new();
    let mut merged = vec![false; regions.len()];
    
    for i in 0..regions.len() {
        if merged[i] {
            continue;
        }
        
        let mut current_region = regions[i];
        merged[i] = true;
        
        let mut merged_any = true;
        while merged_any {
            merged_any = false;
            
            for j in 0..regions.len() {
                if merged[j] || i == j {
                    continue;
                }
                
                if current_region.overlaps(&regions[j]) {
                    current_region = current_region.merge(&regions[j]);
                    merged[j] = true;
                    merged_any = true;
                }
            }
        }
        
        result.push(current_region);
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{RgbaImage, ImageBuffer};
    
    // テスト用の画像を作成
    fn create_test_image(width: u32, height: u32, color: Rgba<u8>) -> DynamicImage {
        let mut img = ImageBuffer::new(width, height);
        for y in 0..height {
            for x in 0..width {
                img.put_pixel(x, y, color);
            }
        }
        DynamicImage::ImageRgba8(img)
    }
    
    // テスト用のCapturedImageを作成
    fn create_test_captured_image(width: u32, height: u32, color: Rgba<u8>) -> CapturedImage {
        let image = create_test_image(width, height, color);
        CapturedImage::new(image, 0)
    }
    
    #[test]
    fn test_rectangle_overlap() {
        let r1 = Rectangle::new(0, 0, 10, 10);
        let r2 = Rectangle::new(5, 5, 10, 10);
        let r3 = Rectangle::new(20, 20, 10, 10);
        
        assert!(r1.overlaps(&r2));
        assert!(r2.overlaps(&r1));
        assert!(!r1.overlaps(&r3));
        assert!(!r3.overlaps(&r1));
    }
    
    #[test]
    fn test_rectangle_merge() {
        let r1 = Rectangle::new(0, 0, 10, 10);
        let r2 = Rectangle::new(5, 5, 10, 10);
        
        let merged = r1.merge(&r2);
        assert_eq!(merged.x, 0);
        assert_eq!(merged.y, 0);
        assert_eq!(merged.width, 15);
        assert_eq!(merged.height, 15);
    }
    
    #[test]
    fn test_diff_calculator_first_image() {
        let mut calculator = DiffCalculator::default();
        let image = create_test_captured_image(100, 100, Rgba([255, 0, 0, 255]));
        
        let result = calculator.calculate(&image);
        
        assert_eq!(result.changed_regions.len(), 1);
        assert_eq!(result.changed_regions[0].x, 0);
        assert_eq!(result.changed_regions[0].y, 0);
        assert_eq!(result.changed_regions[0].width, 100);
        assert_eq!(result.changed_regions[0].height, 100);
        assert_eq!(result.change_ratio, 1.0);
    }
    
    #[test]
    fn test_diff_calculator_no_change() {
        let mut calculator = DiffCalculator::default();
        let image1 = create_test_captured_image(100, 100, Rgba([255, 0, 0, 255]));
        let image2 = create_test_captured_image(100, 100, Rgba([255, 0, 0, 255]));
        
        let _ = calculator.calculate(&image1);
        let result = calculator.calculate(&image2);
        
        assert_eq!(result.changed_regions.len(), 0);
        assert_eq!(result.change_ratio, 0.0);
    }
    
    #[test]
    fn test_diff_calculator_size_change() {
        let mut calculator = DiffCalculator::default();
        let image1 = create_test_captured_image(100, 100, Rgba([255, 0, 0, 255]));
        let image2 = create_test_captured_image(200, 100, Rgba([255, 0, 0, 255]));
        
        let _ = calculator.calculate(&image1);
        let result = calculator.calculate(&image2);
        
        assert_eq!(result.changed_regions.len(), 1);
        assert_eq!(result.changed_regions[0].width, 200);
        assert_eq!(result.change_ratio, 1.0);
    }
    
    #[test]
    fn test_pixel_diff() {
        let p1 = Rgba([100, 100, 100, 255]);
        let p2 = Rgba([120, 120, 120, 255]);
        let p3 = Rgba([100, 100, 100, 100]); // アルファ値が異なる
        
        assert_eq!(pixel_diff(p1, p1), 0);
        assert_eq!(pixel_diff(p1, p2), 20);
        // アルファ値の差は無視
        assert_eq!(pixel_diff(p1, p3), 0);
    }
}