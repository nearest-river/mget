
use futures::future;
use std::{
  path::PathBuf,
  sync::LazyLock
};

use reqwest::{
  Url,
  Client,
  IntoUrl,
  Response
};

use indicatif::{
  ProgressBar,
  MultiProgress,
  ProgressStyle
};

use tokio::{
  sync::Mutex,
  fs::OpenOptions,
  task::JoinHandle,
  io::AsyncWriteExt,
  io::{
    Error,
    ErrorKind
  }
};


#[derive(Default)]
pub struct Downloader {
  progress_bars: MultiProgress,
  tasks: Mutex<Vec<JoinHandle<anyhow::Result<()>>>>
}

macro_rules! download_task {
  ($this:expr,$url:expr)=> {
    tokio::spawn(download($this.progress_bars.clone(),$url))
  };
}

impl Downloader {
  #[inline]
  pub fn new()-> Self {
    Self::default()
  }

  #[inline]
  pub async fn add_to_queue(&self,url: Url) {
    let mut tasks=self.tasks.lock().await;
    tasks.push(download_task!(self,url))
  }

  #[inline]
  pub async fn extent_queue<I: ExactSizeIterator<Item=Url>>(&self,iter: I) {
    let mut tasks=self.tasks.lock().await;
    tasks.reserve(iter.len());
    tasks.extend(iter.map(|url| download_task!(self,url)));
  }

  pub async fn await_all(self)-> anyhow::Result<()> {
    let iter=future::join_all(self.tasks.into_inner()).await
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
}


async fn download<T: IntoUrl>(progress_bars: MultiProgress,url: T)-> anyhow::Result<()> {
  let url=url.into_url()?;
  let path: PathBuf=percent_encoding::percent_decode_str(url.path())
  .decode_utf8_lossy()
  .into_owned()
  .into();
  // SAFETY: trust me bro. (its already been filterred 69 times)
  let file_name=unsafe {
    path.file_name()
    .unwrap_unchecked()
  };

  let res=get(url).await?.bytes().await?;
  let bar=progress_bars.add(
    ProgressBar::new(res.len() as _)
    .with_style(progress_style())
  );

  let mut file=OpenOptions::new()
  .create(true)
  .write(true)
  .read(true)
  .open(file_name)
  .await?;

  let mut buf=res.as_ref();
  while !buf.is_empty() {
    match file.write(buf).await {
      Ok(0)=> Err(Error::new(ErrorKind::WriteZero, "failed to write whole buffer"))?,
      Ok(n)=> {
        buf=&buf[n..];
        bar.inc(n as _);
      },
      Err(ref e) if e.kind()==ErrorKind::Interrupted=> (),
      Err(e)=> return Err(e)?
    }
  }

  bar.finish_with_message(format!("downloaded {file_name:#?}"));
  Ok(())
}

#[inline]
async fn get<T: IntoUrl>(url: T)-> reqwest::Result<Response> {
  static CLIENT: LazyLock<Client>=LazyLock::new(|| Client::new());

  CLIENT.get(url)
  .send()
  .await
}

#[inline(always)]
fn progress_style()-> ProgressStyle {
  ProgressStyle::with_template(
    "[{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})"
  )
  .unwrap()
  .progress_chars("#>-")
}

