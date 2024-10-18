
use regex::Regex;
use std::{
  error::Error,
  str::FromStr,
  hash::RandomState,
  collections::HashSet,
  fmt::{
    self,
    Display,
    Formatter
  }
};


pub trait PatternExt {
  fn pattern(&self)-> Regex;
}

#[derive(Debug,Default,Clone)]
pub enum DocType {
  #[default]
  Video,
  Image,
  Audio,
  Pdf,
  Text,
  DiskImage,
  Zip,
  Other
}

#[derive(Debug)]
pub struct ParseTargetError(Box<str>);

impl FromStr for DocType {
  type Err=ParseTargetError;
  fn from_str(s: &str)-> Result<Self,Self::Err> {
    match s {
      "mkv"|"mp4"|"webm"=> Ok(Self::Video),
      "jpeg"|"jpg"|"png"|"svg"|"webp"|"gif"=> Ok(Self::Image),
      "mp3"|"m4a"|"aac"=> Ok(Self::Audio),
      "pdf"=> Ok(Self::Pdf),
      "txt"=> Ok(Self::Text),
      "iso"|"img"=> Ok(Self::DiskImage),
      "zip"|"gz"|"xz"|"7z"|"rar"=> Ok(Self::Zip),
      "*"=> Ok(Self::Other),
      _=> Err(From::from(s))
    }
  }
}

static DOC_TYPE_VIDEO: &'static [&'static str]=&["mkv","mp4","webm"];
static DOC_TYPE_IMAGE: &'static [&'static str]=&["jpeg","jpg","png","svg","webp","gif"];
static DOC_TYPE_AUDIO: &'static [&'static str]=&["mp3","m4a","aac"];
static DOC_TYPE_PDF: &'static [&'static str]=&["pdf"];
static DOC_TYPE_TEXT: &'static [&'static str]=&["txt"];
static DOC_TYPE_DISK_IMAGE: &'static [&'static str]=&["iso","img"];
static DOC_TYPE_ZIP: &'static [&'static str]=&["zip","gz","xz","7z","rar"];
static DOC_TYPE_OTHER: &'static [&'static str]=&["*"];

impl DocType {
  pub fn exts(&self)-> &[&str] {
    match self {
      Self::Video=> DOC_TYPE_VIDEO,
      Self::Image=> DOC_TYPE_IMAGE,
      Self::Audio=> DOC_TYPE_AUDIO,
      Self::Pdf=> DOC_TYPE_PDF,
      Self::Text=> DOC_TYPE_TEXT,
      Self::DiskImage=> DOC_TYPE_DISK_IMAGE,
      Self::Zip=> DOC_TYPE_ZIP,
      Self::Other=> DOC_TYPE_OTHER,
    }
  }
}

impl PatternExt for DocType {
  fn pattern(&self)-> Regex {
    // SAFETY: The formed regices should be okay.
    unsafe {
      // it's nothing but r"\.(ext1|ext2)$"
      Regex::new(&format!(r"\.({})$",self.exts().join("|")))
      .unwrap_unchecked()
    }
  }
}

impl<I: AsRef<[DocType]>> PatternExt for I {
  fn pattern(&self)-> Regex {
    let mut doc_types=Vec::<&str>::new();
    for doc_type in self.as_ref() {
      doc_types.extend_from_slice(doc_type.exts());
    }

    let pattern=HashSet::<_,RandomState>::from_iter(doc_types)
    .into_iter()
    .collect::<Box<[&str]>>()
    .join("|");

    // SAFETY: The formed regices should be okay.
    unsafe {
      // it's nothing but r"\.(ext1|ext2)$"
      Regex::new(&format!(r"\.({pattern})$"))
      .unwrap_unchecked()
    }
  }
}


impl Error for ParseTargetError {}

impl Display for ParseTargetError {
  #[inline]
  fn fmt(&self,f: &mut Formatter<'_>)-> fmt::Result {
    writeln!(f,"{} isn't a valid document type.",self.0)
  }
}

impl<S: AsRef<str>> From<S> for ParseTargetError {
  #[inline]
  fn from(value: S)-> Self {
    ParseTargetError(value.as_ref().into())
  }
}

