// src/util/tiling.rs

use crate::types::TileRect;

/// Splits the source texture of a given size into multiple smaller tiles according to the max_size limit.
///
/// The number of tiles is calculated independently for each axis (width, height) to prevent
/// creating unnecessary rectangular tiles for images with extreme aspect ratios and to find the optimal layout.
///
/// # Arguments
/// * `width` - Original width of the texture.
/// * `height` - Original height of the texture.
/// * `max_size` - Maximum dimension allowed for a single tile.
///
/// # Returns
/// A vector of `TileRect` representing the split tiles.
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
