
mod dom_ext;
mod doc_type;
mod downloader;

use reqwest::Url;
use clap::Parser;
use dom_ext::DomExt;
use futures::future;
use tokio::task::JoinHandle;



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
  let mut tasks=Vec::new();

  for url in urls {
    let path=url.path();
    if pattern.is_match(path) {
      tasks.push(tokio::spawn(downloader::download(url)));
      continue;
    }

    let html=reqwest::get(url)
    .await?
    .text()
    .await?;
    let _extracted_urls=tl::parse(&html,Default::default())?
    .extract_urls(&pattern);
  }


  await_tasks(tasks).await
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

async fn await_tasks<T>(tasks: Vec<JoinHandle<anyhow::Result<T>>>)-> anyhow::Result<()> {
  let iter=future::join_all(tasks).await
  .into_iter()
  .filter(Result::is_ok);

  for task in iter {
    // SAFETY: trust me bro. (I just filterred it in the previous line.. cant you see?.. idiot)
    let res=unsafe {
      task.unwrap_unchecked()
    };

    if let Err(err)=res {
      tracing::error!("{err}");
    }
  }

  Ok(())
}

