//! 글자 크기 한 곳. macroquad는 처음 보는 (글자, 크기)를 그릴 때 글꼴 아틀라스를 키우며 **옛 텍스처를 지운다** —
//! 같은 프레임에 먼저 쌓인 글자 그리기가 지워진 텍스처를 가리켜 WebGL이 `glBindTexture … already deleted`를 낸다
//! (macroquad 0.4.16 `text/atlas.rs` `texture()`). 그래서 쓰는 크기를 몇 개로 정해 두고, 그리기 전에 ASCII를 다 구워 둔다.
//! 크기 키는 `ceil(크기 × DPI)`라 DPI가 바뀌면 다시 굽는다.
use std::sync::atomic::{AtomicU32, Ordering};

use macroquad::prelude::*;

/// 화면 글자 크기(논리 px). 바뀌는 크기(폰의 랩 줄·버튼 글)는 `fit`으로 이 중 하나에 맞춘다.
pub const SIZES: [f32; 10] = [10.0, 12.0, 14.0, 16.0, 18.0, 20.0, 24.0, 28.0, 32.0, 40.0];

/// `want` 이하에서 가장 큰 정한 크기(가장 작은 것보다 작으면 가장 작은 것).
pub fn fit(want: f32) -> f32 {
    SIZES
        .iter()
        .copied()
        .rev()
        .find(|s| *s <= want)
        .unwrap_or(SIZES[0])
}

static WARM_DPI: AtomicU32 = AtomicU32::new(0);

/// 프레임마다 그리기 전에. DPI가 처음이거나 바뀌었을 때만 굽는다(ASCII 95자 × 크기 열).
pub fn warm() {
    let dpi = screen_dpi_scale();
    if WARM_DPI.swap(dpi.to_bits(), Ordering::Relaxed) == dpi.to_bits() {
        return;
    }
    let chars: Vec<char> = (32u8..127).map(char::from).collect();
    let font = get_default_font();
    for s in SIZES {
        font.populate_font_cache(&chars, (s * dpi).ceil() as u16);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dynamic_sizes_snap_down_to_a_warmed_size() {
        assert_eq!(fit(16.0), 16.0);
        assert_eq!(fit(23.4), 20.0);
        assert_eq!(fit(5.0), 10.0);
        assert_eq!(fit(100.0), 40.0);
        for s in SIZES {
            assert_eq!(fit(s), s);
        }
    }
}
