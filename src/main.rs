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


#[macro_use]
mod dom_ext;
mod doc_type;
mod downloader;

use clap::Parser;
use reqwest::Url;
use dom_ext::DomExt;
use std::path::Path;
use downloader::Downloader;
use futures::{
  stream,
  StreamExt
};



#[derive(Parser)]
struct App {
  urls: Vec<String>,
  #[arg(long,short,alias="out",default_value=".")]
  out_dir: Box<Path>,
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
      let path=percent_encode_path!(url);

      // SAFETY: trust me bro. (its already been filterred 69 times)
      downloader.add_to_queue(
        url,
        app.out_dir.join(path.file_name().unwrap())
      ).await;
      return Ok(());
    }

    let html=reqwest::get(url)
    .await?
    .text()
    .await?;

    let extracted_urls=tl::parse(&html,Default::default())?
    .extract_urls(&pattern,&app.out_dir);
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



