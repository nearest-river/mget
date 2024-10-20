
use regex::Regex;
use std::ops::Deref;


static DOC_TYPE_VIDEO: &'static [&'static str]=&["mkv","mp4","webm"];
static DOC_TYPE_IMAGE: &'static [&'static str]=&["jpeg","jpg","png","svg","webp","gif"];
static DOC_TYPE_AUDIO: &'static [&'static str]=&["mp3","m4a","aac"];
static DOC_TYPE_PDF: &'static [&'static str]=&["pdf"];
static DOC_TYPE_TEXT: &'static [&'static str]=&["txt"];
static DOC_TYPE_DISK_IMAGE: &'static [&'static str]=&["iso","img"];
static DOC_TYPE_ZIP: &'static [&'static str]=&["zip","gz","xz","7z","rar"];


pub fn parse<S: Deref<Target=str>>(doc_types: &[S])-> Regex {
  let mut exts=Vec::<&'static str>::new();

  for ext in doc_types {
    if ext.eq("*") {
      // SAFETY: its just clearing off the vector of `&'static str`.
      // `&'static str`s dont need to be destructed.
      unsafe {
        exts.set_len(1);
      }
      exts[0]="*";
      break;
    }

    if let Some(extensions)=map_aliases(ext) {
      exts.extend_from_slice(extensions);
      continue;
    }

    tracing::warn!("warning: {} isn't a valid alias",ext.deref());
  }


  // SAFETY: The formed regices should be okay.
  unsafe {
    // it's nothing but r"\.(ext1|ext2)$"
    Regex::new(&format!(r"\.({})$",exts.join("|")))
    .unwrap_unchecked()
  }
}

#[inline]
fn map_aliases(s: &str)-> Option<&'static [&'static str]> {
  match s {
    "video"|"vid"=> Some(DOC_TYPE_VIDEO),
    "image"|"img"=> Some(DOC_TYPE_IMAGE),
    "audio"|"aud"=> Some(DOC_TYPE_AUDIO),
    "pdf"=> Some(DOC_TYPE_PDF),
    "text"=> Some(DOC_TYPE_TEXT),
    "disk-image"=> Some(DOC_TYPE_DISK_IMAGE),
    "zip-file"=> Some(DOC_TYPE_ZIP),
    _=> None
  }
}
