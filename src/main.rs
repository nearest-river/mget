
mod dom_ext;
mod doc_type;
mod downloader;

use reqwest::Url;
use clap::Parser;
use dom_ext::DomExt;
use downloader::Downloader;



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
  let mut downloader=Downloader::new();

  for url in urls {
    let path=url.path();
    if pattern.is_match(path) {
      downloader.add_to_queue(url);
      continue;
    }

    let html=reqwest::get(url)
    .await?
    .text()
    .await?;

    let extracted_urls=tl::parse(&html,Default::default())?
    .extract_urls(&pattern);
    downloader.extent_queue(extracted_urls.into_iter());
  }

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


