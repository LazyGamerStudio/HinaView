// src/util/tiling.rs

use crate::types::TileRect;

/// 주어진 크기의 원본 텍스처를 max_size 한계에 맞추어 여러 개의 작은 타일로 분할합니다.
/// 각 축(가로, 세로)에 대해 독립적으로 개수를 산정하여, 특정 축만 긴 이미지의 경우
/// 불필요한 직사각형 타일이 많이 만들어지는 것을 방지하고 최적의 배치를 찾습니다.
pub fn compute_tiles(width: u32, height: u32, max_size: u32) -> Vec<TileRect> {
    if width <= max_size && height <= max_size {
        return vec![TileRect {
            x: 0,
            y: 0,
            width,
            height,
        }];
    }

    let cols = width.div_ceil(max_size);
    let rows = height.div_ceil(max_size);
    let mut tiles = Vec::with_capacity((cols * rows) as usize);

    for row in 0..rows {
        let y = row * max_size;
        let th = if row == rows - 1 {
            height - y
        } else {
            max_size
        };

        for col in 0..cols {
            let x = col * max_size;
            let tw = if col == cols - 1 { width - x } else { max_size };

            tiles.push(TileRect {
                x,
                y,
                width: tw,
                height: th,
            });
        }
    }

    tiles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_tiles_single() {
        let tiles = compute_tiles(800, 600, 4096);
        assert_eq!(tiles.len(), 1);
        assert_eq!(
            tiles[0],
            TileRect {
                x: 0,
                y: 0,
                width: 800,
                height: 600
            }
        );
    }

    #[test]
    fn test_compute_tiles_horizontal_panorama() {
        let tiles = compute_tiles(10000, 1000, 4096);
        assert_eq!(tiles.len(), 3); // 10000 / 4096 = 2.44 -> 3 cols, 1 row

        assert_eq!(
            tiles[0],
            TileRect {
                x: 0,
                y: 0,
                width: 4096,
                height: 1000
            }
        );
        assert_eq!(
            tiles[1],
            TileRect {
                x: 4096,
                y: 0,
                width: 4096,
                height: 1000
            }
        );
        assert_eq!(
            tiles[2],
            TileRect {
                x: 8192,
                y: 0,
                width: 10000 - 8192,
                height: 1000
            }
        );
    }

    #[test]
    fn test_compute_tiles_large_square() {
        let tiles = compute_tiles(10000, 10000, 4096);
        assert_eq!(tiles.len(), 9); // 3 cols, 3 rows

        // Check some boundaries
        assert_eq!(
            tiles[0],
            TileRect {
                x: 0,
                y: 0,
                width: 4096,
                height: 4096
            }
        );
        assert_eq!(
            tiles[4],
            TileRect {
                x: 4096,
                y: 4096,
                width: 4096,
                height: 4096
            }
        );
        assert_eq!(
            tiles[8],
            TileRect {
                x: 8192,
                y: 8192,
                width: 10000 - 8192,
                height: 10000 - 8192
            }
        );
    }
}
