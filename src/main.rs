
mod doc_type;
use clap::Parser;
use tracing::{
  info,
  warn
};

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
  init_logger();
  let app=App::parse();
  let urls=app.urls;
  let pattern=doc_type::parse(&app.target_docs);

  for url in urls {
    let path=url.path();
    if pattern.is_match(path) {
      download_vid(url).await?;
      continue;
    }
    if path.ends_with('/') {
      warn!("invalid path: {path}\nSkipped..");
      continue;
    }


    let html=reqwest::get(url.as_str()).await?.text().await?;
    let _dom=tl::parse(&html,Default::default())?;
    info!("for {url}:\n{html}");
  }


  Ok(())
}

fn init_logger() {
  tracing_subscriber::FmtSubscriber::builder()
  .compact()
  .with_line_number(false)
  .without_time()
  .with_level(false)
  .with_target(false)
  .init();
}


async fn download_vid<T: IntoUrl>(url: T)-> anyhow::Result<()> {
  let res=reqwest::get(url).await?;

  info!("{res:#?}");
  Ok(())
}
