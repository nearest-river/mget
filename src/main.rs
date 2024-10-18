
mod target;
use target::*;
use clap::Parser;

use reqwest::{
  Url,
  IntoUrl
};


#[derive(Parser)]
struct App {
  urls: Vec<Url>,
  target_docs: Vec<DocType>
}



#[tokio::main]
async fn main()-> anyhow::Result<()> {
  let app=App::parse();
  let urls=app.urls;
  let pattern=app.target_docs.pattern();

  for url in urls {
    let path=url.path();
    if pattern.is_match(path) {
      download_vid(url).await?;
      continue;
    }
    if !path.ends_with('/') {
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
