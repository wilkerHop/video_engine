use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Asset types that can be loaded
#[derive(Debug, Clone)]
pub enum Asset {
    Image(ImageAsset),
    Video(VideoAsset),
    Font(FontAsset),
}

/// Image asset
#[derive(Debug, Clone)]
pub struct ImageAsset {
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
}

/// Video asset (placeholder for now, will use FFmpeg later)
#[derive(Debug, Clone)]
pub struct VideoAsset {
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub fps: f32,
    pub duration: f32,
}

/// Font asset (placeholder for now)
#[derive(Debug, Clone)]
pub struct FontAsset {
    pub path: PathBuf,
    pub data: Vec<u8>,
}

/// Asset loader that manages loading and caching of assets
pub struct AssetLoader {
    assets: HashMap<PathBuf, Asset>,
    base_path: PathBuf,
}

impl AssetLoader {
    /// Create a new asset loader with a base path for resolving relative paths
    pub fn new(base_path: impl AsRef<Path>) -> Self {
        Self {
            assets: HashMap::new(),
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    /// Load an image asset (stub for now)
    pub fn load_image(&mut self, path: &Path) -> Result<&ImageAsset> {
        let full_path = self.resolve_path(path);

        if !self.assets.contains_key(&full_path) {
            // Verify file exists
            if !full_path.exists() {
                anyhow::bail!("Image file not found: {}", full_path.display());
            }

            // TODO: Use actual image library to load pixels
            // For now, create a stub with placeholder dimensions
            let asset = Asset::Image(ImageAsset {
                path: full_path.clone(),
                width: 1920,
                height: 1080,
            });

            self.assets.insert(full_path.clone(), asset);
        }

        match self.assets.get(&full_path).unwrap() {
            Asset::Image(img) => Ok(img),
            _ => anyhow::bail!("Asset is not an image"),
        }
    }

    /// Load a video asset (stub implementation)
    pub fn load_video(&mut self, path: &Path) -> Result<&VideoAsset> {
        let full_path = self.resolve_path(path);

        if !self.assets.contains_key(&full_path) {
            // Verify file exists
            if !full_path.exists() {
                anyhow::bail!("Video file not found: {}", full_path.display());
            }

            // TODO: Use FFmpeg to get actual video properties
            // For now, create a stub
            let asset = Asset::Video(VideoAsset {
                path: full_path.clone(),
                width: 1920,
                height: 1080,
                fps: 30.0,
                duration: 10.0,
            });

            self.assets.insert(full_path.clone(), asset);
        }

        match self.assets.get(&full_path).unwrap() {
            Asset::Video(vid) => Ok(vid),
            _ => anyhow::bail!("Asset is not a video"),
        }
    }

    /// Load a font asset
    pub fn load_font(&mut self, path: &Path) -> Result<&FontAsset> {
        let full_path = self.resolve_path(path);

        if !self.assets.contains_key(&full_path) {
            let data = std::fs::read(&full_path)
                .with_context(|| format!("Failed to load font: {}", full_path.display()))?;

            let asset = Asset::Font(FontAsset {
                path: full_path.clone(),
                data,
            });

            self.assets.insert(full_path.clone(), asset);
        }

        match self.assets.get(&full_path).unwrap() {
            Asset::Font(font) => Ok(font),
            _ => anyhow::bail!("Asset is not a font"),
        }
    }

    /// Resolve a path relative to the base path
    fn resolve_path(&self, path: &Path) -> PathBuf {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.base_path.join(path)
        }
    }

    /// Get statistics about loaded assets
    pub fn stats(&self) -> AssetStats {
        let mut images = 0;
        let mut videos = 0;
        let mut fonts = 0;

        for asset in self.assets.values() {
            match asset {
                Asset::Image(_) => images += 1,
                Asset::Video(_) => videos += 1,
                Asset::Font(_) => fonts += 1,
            }
        }

        AssetStats {
            total: self.assets.len(),
            images,
            videos,
            fonts,
        }
    }

    /// Clear all loaded assets from memory
    pub fn clear(&mut self) {
        self.assets.clear();
    }
}

/// Statistics about loaded assets
#[derive(Debug, Clone)]
pub struct AssetStats {
    pub total: usize,
    pub images: usize,
    pub videos: usize,
    pub fonts: usize,
}

impl std::fmt::Display for AssetStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Total: {}, Images: {}, Videos: {}, Fonts: {}",
            self.total, self.images, self.videos, self.fonts
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_asset_loader() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // Create a test file (stub image)
        let img_path = base_path.join("test.png");
        fs::write(&img_path, b"fake image data").unwrap();

        let mut loader = AssetLoader::new(base_path);
        let result = loader.load_image(Path::new("test.png"));
        assert!(result.is_ok());

        let stats = loader.stats();
        assert_eq!(stats.images, 1);
        assert_eq!(stats.total, 1);
    }

    #[test]
    fn test_load_nonexistent_image() {
        let temp_dir = TempDir::new().unwrap();
        let mut loader = AssetLoader::new(temp_dir.path());

        let result = loader.load_image(Path::new("nonexistent.png"));
        assert!(result.is_err());
    }

    #[test]
    fn test_asset_caching() {
        let temp_dir = TempDir::new().unwrap();
        let img_path = temp_dir.path().join("cached.png");
        fs::write(&img_path, b"data").unwrap();

        let mut loader = AssetLoader::new(temp_dir.path());

        // Load once
        loader.load_image(Path::new("cached.png")).unwrap();
        let stats1 = loader.stats();

        // Load again - should use cache
        loader.load_image(Path::new("cached.png")).unwrap();
        let stats2 = loader.stats();

        assert_eq!(stats1.total, stats2.total); // Should be same (cached)
        assert_eq!(stats2.total, 1);
    }

    #[test]
    fn test_resolve_path_absolute() {
        let loader = AssetLoader::new("/base");
        let abs_path = PathBuf::from("/absolute/path.png");
        let resolved = loader.resolve_path(&abs_path);
        assert_eq!(resolved, abs_path);
    }

    #[test]
    fn test_resolve_path_relative() {
        let loader = AssetLoader::new("/base");
        let rel_path = Path::new("relative/path.png");
        let resolved = loader.resolve_path(rel_path);
        assert_eq!(resolved, PathBuf::from("/base/relative/path.png"));
    }

    #[test]
    fn test_load_video() {
        let temp_dir = TempDir::new().unwrap();
        let vid_path = temp_dir.path().join("test.mp4");
        fs::write(&vid_path, b"fake video").unwrap();

        let mut loader = AssetLoader::new(temp_dir.path());
        let result = loader.load_video(Path::new("test.mp4"));
        assert!(result.is_ok());

        let video = result.unwrap();
        assert_eq!(video.width, 1920);
        assert_eq!(video.height, 1080);
        assert_eq!(video.fps, 30.0);
    }

    #[test]
    fn test_load_font() {
        let temp_dir = TempDir::new().unwrap();
        let font_path = temp_dir.path().join("font.ttf");
        fs::write(&font_path, b"fake font data").unwrap();

        let mut loader = AssetLoader::new(temp_dir.path());
        let result = loader.load_font(Path::new("font.ttf"));
        assert!(result.is_ok());

        let stats = loader.stats();
        assert_eq!(stats.fonts, 1);
        assert_eq!(stats.total, 1);
    }

    #[test]
    fn test_clear_assets() {
        let temp_dir = TempDir::new().unwrap();
        let img_path = temp_dir.path().join("test.png");
        fs::write(&img_path, b"data").unwrap();

        let mut loader = AssetLoader::new(temp_dir.path());
        loader.load_image(Path::new("test.png")).unwrap();

        assert_eq!(loader.stats().total, 1);

        loader.clear();
        assert_eq!(loader.stats().total, 0);
    }

    #[test]
    fn test_asset_stats_display() {
        let stats = AssetStats {
            total: 10,
            images: 5,
            videos: 3,
            fonts: 2,
        };
        let display = format!("{}", stats);
        assert!(display.contains("Total: 10"));
        assert!(display.contains("Images: 5"));
        assert!(display.contains("Videos: 3"));
        assert!(display.contains("Fonts: 2"));
    }
}
