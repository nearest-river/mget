
mod dom_ext;
mod doc_type;
mod downloader;

use clap::Parser;
use reqwest::Url;
use dom_ext::DomExt;
use downloader::Downloader;
use futures::{
  stream,
  StreamExt
};



#[derive(Parser)]
struct App {
  urls: Vec<String>,
  #[arg(long,alias="target",default_value="vid")]
  target_docs: Vec<Box<str>>
}


#[tokio::main]
async fn main()-> anyhow::Result<()> {
  init_logger();
  let app=App::parse();
  let pattern=doc_type::parse(&app.target_docs);
  let downloader=Downloader::new();

  stream::iter(app.urls)
  .map(|url| Url::parse(&url))
  .map(|url| async {
    let url=url?;
    let path=url.path();
    if !path.ends_with('/') {
      downloader.add_to_queue(url).await;
      return Ok(());
    }

    let html=reqwest::get(url)
    .await?
    .text()
    .await?;

    let extracted_urls=tl::parse(&html,Default::default())?
    .extract_urls(&pattern);
    downloader.extent_queue(extracted_urls.into_iter()).await;
    anyhow::Ok(())
  })
  .for_each_concurrent(None,|res| async {
    if let Err(err)=res.await {
      tracing::error!("{err}");
    }
  }).await;

  downloader.await_all().await
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


