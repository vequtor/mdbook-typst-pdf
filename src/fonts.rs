use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use fontdb::{Database, Source};
use typst::text::{Font, FontBook, FontInfo};
use typst_timing::TimingScope;

/// Searches for fonts.
pub struct FontSearcher {
  /// Metadata about all discovered fonts.
  pub book: FontBook,
  /// Slots that the fonts are loaded into.
  pub fonts: Vec<FontSlot>,
}

/// Holds details about the location of a font and lazily the font itself.
pub struct FontSlot {
  /// The path at which the font can be found on the system.
  path: PathBuf,
  /// The index of the font in its collection. Zero if the path does not point
  /// to a collection.
  index: u32,
  /// The lazily loaded font.
  font: OnceLock<Option<Font>>,
}

impl FontSlot {
  /// Get the font for this slot.
  pub fn get(&self) -> Option<Font> {
    self
      .font
      .get_or_init(|| {
        let _scope = TimingScope::new("load font", None);
        let data = fs::read(&self.path).ok()?.into();
        Font::new(data, self.index)
      })
      .clone()
  }
}

impl FontSearcher {
  /// Create a new, empty system searcher.
  pub fn new() -> Self {
    Self {
      book: FontBook::new(),
      fonts: vec![],
    }
  }

  /// Search everything that is available.
  pub fn search(&mut self, font_paths: &[PathBuf]) {
    let mut db = Database::new();

    // Font paths have highest priority.
    for path in font_paths {
      db.load_fonts_dir(path);
    }

    // System fonts have second priority.
    db.load_system_fonts();

    for face in db.faces() {
      let path = match &face.source {
        Source::File(path) | Source::SharedFile(path, _) => path,
        // We never add binary sources to the database, so there
        // shouln't be any.
        Source::Binary(_) => continue,
      };

      let info = db
        .with_face_data(face.id, FontInfo::new)
        .expect("database must contain this font");

      if let Some(info) = info {
        self.book.push(info);
        self.fonts.push(FontSlot {
          path: path.clone(),
          index: face.index,
          font: OnceLock::new(),
        });
      }
    }
  }
}
