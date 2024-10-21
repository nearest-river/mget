
use core::str;
use regex::Regex;
use reqwest::Url;
use tl::{
  Node,
  VDom
};



pub trait DomExt {
  fn extract_urls(&self,pattern: &Regex)-> Vec<Url>;
}

impl DomExt for VDom<'_> {
  fn extract_urls(&self,pattern: &Regex)-> Vec<Url> {
    let mut urls=Vec::new();
    // SAFETY: `a` is a html tag so the query-selector is always valid
    let nodes=unsafe {
      self.query_selector("a").unwrap_unchecked()
    };

    for node_handle in nodes {
      let herf=node_handle.get(self.parser())
      .unwrap()
      .extract_herf(pattern);
      let url=match herf {
        Some(url)=> url,
        _=> continue
      };

      // SAFETY: the filtered links are absolute paths
      urls.push(unsafe {
        Url::parse(url).unwrap_unchecked()
      });
    }

    urls
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
    let mut map=HashMap::<OsString,Vec<String>>::new();

    for file in fs::read_dir("./assets")? {
      let file=file?;
      let html=fs::read_to_string(file.path())?;
      let dom=tl::parse(&html,Default::default())?;
      let extracted_urls=dom.extract_urls(&Regex::new(r"\.(mp4|mkv)$")?)
      .into_iter()
      .map(|url| url.into())
      .collect::<Vec<_>>();

      map.insert(file.file_name(),extracted_urls);
    }

    println!("{map:#?}");
    Ok(())
  }
}

