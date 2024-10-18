
use clap::Parser;
use regex::Regex;

use reqwest::{
  Url,
  IntoUrl
};


#[derive(Parser)]
struct App {
  urls: Vec<Url>,
  #[arg(long,alias="target",default_value="vid")]
  target_docs: Vec<Box<str>>
}



#[tokio::main]
async fn main()-> anyhow::Result<()> {
  let app=App::parse();
  let urls=app.urls;
  // SAFETY: The formed regices should be okay.
  let pattern=unsafe {
    // it's nothing but r"\.(ext1|ext2)$"
    Regex::new(&format!(r"\.({})$",app.target_docs.join("|")))
    .unwrap_unchecked()
  };

  for url in urls {
    let path=url.path();
    if pattern.is_match(path) {
      download_vid(url).await?;
      continue;
    }
    if path.ends_with('/') {
      log::warn!("invalid path: {path}\nSkipped..");
      continue;
    }


    let html=reqwest::get(url.as_str()).await?.text().await?;
    let _dom=tl::parse(&html,Default::default())?;
    println!("for {url}:\n{html}");
  }


  Ok(())
}


async fn download_vid<T: IntoUrl>(url: T)-> anyhow::Result<()> {
  let res=reqwest::get(url).await?;

  println!("{res:#?}");
  Ok(())
}
