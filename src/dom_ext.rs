// This file is part of mget
//
// mget is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 2 of the License, or
// (at your option) any later version.
//
// mger is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with mget. If not, see <http://www.gnu.org/licenses/>.

use core::str;
use regex::Regex;
use reqwest::Url;
use tl::{
  Node,
  VDom
};

use std::path::{
  Path,
  PathBuf
};


#[macro_export]
macro_rules! percent_encode_path {
  ($url:expr)=> {
    std::path::PathBuf::from(
      percent_encoding::percent_decode_str($url.path())
      .decode_utf8_lossy()
      .into_owned()
    )
  };
}


pub trait DomExt {
  fn extract_urls<P: AsRef<Path>>(&self,pattern: &Regex,dir: P)-> Vec<(Url,PathBuf)>;
}

impl DomExt for VDom<'_> {
  fn extract_urls<P: AsRef<Path>>(&self,pattern: &Regex,dir: P)-> Vec<(Url,PathBuf)> {
    // SAFETY: `a` is a html tag so the query-selector is always valid
    self.query_selector("a")
    .unwrap()
    .filter_map(|node_handle| {
      let herf=node_handle.get(self.parser())
      .unwrap()
      .extract_herf(pattern);
      let url=match herf {
        Some(url)=> url,
        _=> return None
      };
      // SAFETY: the filtered links are absolute paths
      let url=Url::parse(url).unwrap();
      let path=percent_encode_path!(url);

      // SAFETY: trust me bro. (its already been filterred 69 times)
      Some((url,dir.as_ref().join(path.file_name().unwrap())))
    }).collect::<Vec<_>>()
  }
}

trait NodeExt {
  fn extract_herf(&self,pattern: &Regex)-> Option<&str>;
}

impl NodeExt for Node<'_> {
  #[inline]
  fn extract_herf(&self,pattern: &Regex)-> Option<&str> {
    let anchor=match self {
      Node::Tag(tag)=> tag,
      _=> return None
    };
    // SAFETY: urls aren't usually invalid utf-8
    let url=unsafe {
      str::from_utf8_unchecked(
        anchor.attributes().get("href")??.as_bytes()
      )
    };

    match pattern.is_match(url) {
      true=> Some(url),
      _=> None
    }
  }
}





#[cfg(test)]
mod tests {
  use super::DomExt;
  use regex::Regex;
  use std::{
    fs,
    ffi::OsString,
    collections::HashMap
  };

  #[test]
  fn xd()-> anyhow::Result<()> {
    let mut map=HashMap::<OsString,Vec<(String,_)>>::new();

    for file in fs::read_dir("./assets")? {
      let file=file?;
      let html=fs::read_to_string(file.path())?;
      let dom=tl::parse(&html,Default::default())?;
      let extracted_urls=dom.extract_urls(&Regex::new(r"\.(mp4|mkv)$")?,"")
      .into_iter()
      .map(|(url,path)| (url.into(),path))
      .collect::<Vec<_>>();

      map.insert(file.file_name(),extracted_urls);
    }

    println!("{map:#?}");
    Ok(())
  }
}

