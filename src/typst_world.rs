use crate::Result;
use chrono::{DateTime, Datelike, FixedOffset, Local, Utc};
use typst::{
    diag::FileResult,
    foundations::{Bytes, Datetime},
    syntax::{FileId, Source, VirtualPath},
    text::{Font, FontBook},
    utils::LazyHash,
    Library, World,
};
use typst_kit::fonts::{FontSlot, Fonts};

/// A world that provides access to the operating system.
pub struct SystemWorld {
    /// The input path.
    main: FileId,
    input: Bytes,
    /// Typst's standard library.
    library: LazyHash<Library>,
    /// Metadata about discovered fonts.
    book: LazyHash<FontBook>,
    /// Locations of and storage for lazily loaded fonts.
    fonts: Vec<FontSlot>,
    source: Source,
    now: DateTime<Utc>,
}

impl SystemWorld {
    /// Create a new system world.
    pub fn new(input: Bytes, source: String) -> Result<Self> {
        let main = FileId::new_fake(VirtualPath::new("/input.typ"));

        let library = Library::builder().build();

        let fonts = Fonts::searcher().include_system_fonts(true).search();
        // .search_with(&command.font_args.font_paths);

        Ok(Self {
            main,
            library: LazyHash::new(library),
            book: LazyHash::new(fonts.book),
            input,
            fonts: fonts.fonts,
            source: Source::detached(source),
            now: Utc::now(),
        })
    }
}

impl World for SystemWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        self.main
    }

    fn source(&self, _id: FileId) -> FileResult<Source> {
        Ok(self.source.clone())
    }

    fn file(&self, _id: FileId) -> FileResult<Bytes> {
        Ok(self.input.clone())
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts[index].get()
    }

    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        // The time with the specified UTC offset, or within the local time zone.
        let with_offset = match offset {
            None => self.now.with_timezone(&Local).fixed_offset(),
            Some(hours) => {
                let seconds = i32::try_from(hours).ok()?.checked_mul(3600)?;
                self.now.with_timezone(&FixedOffset::east_opt(seconds)?)
            }
        };

        Datetime::from_ymd(
            with_offset.year(),
            with_offset.month().try_into().ok()?,
            with_offset.day().try_into().ok()?,
        )
    }
}
