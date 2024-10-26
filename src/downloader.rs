
use futures::future;
use std::{
  env,
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
  fs::OpenOptions,
  task::JoinHandle,
  io::AsyncWriteExt,
  io::{
    self,
    Error,
    ErrorKind
  }
};


#[cfg(target_os="android")]
static DOWNLOAD_DIR: &'static str=PathBuf::from("/storage/emulated/0/Download");
#[cfg(not(target_os="android"))]
static DOWNLOAD_DIR: LazyLock<PathBuf>=LazyLock::new(|| {
  let mut path=match env::var("HOME") {
    Ok(home)=> PathBuf::from(home),
    _=> PathBuf::from(env!("HOME"))
  };

  path.push("Download");
  path
});

#[derive(Default)]
pub struct Downloader {
  progress_bars: MultiProgress,
  tasks: Vec<JoinHandle<anyhow::Result<()>>>
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
  pub fn add_to_queue(&mut self,url: Url) {
    self.tasks.push(download_task!(self,url))
  }

  #[inline]
  pub fn extent_queue<I: ExactSizeIterator<Item=Url>>(&mut self,iter: I) {
    self.tasks.reserve(iter.len());
    self.tasks.extend(iter.map(|url| download_task!(self,url)));
  }

  pub async fn await_all(self)-> anyhow::Result<()> {
    let iter=future::join_all(self.tasks).await
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

  let res=get(url).await?;
  let bar=progress_bars.add(
    ProgressBar::new(content_len(&res)?)
    .with_style(progress_style())
  );

  let mut file=OpenOptions::new()
  .create(true)
  .write(true)
  .read(true)
  .open(DOWNLOAD_DIR.join(file_name))
  .await?;

  let bytes=res.bytes().await?;
  let mut buf=bytes.as_ref();
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

  Ok(())
}

#[inline(always)]
fn content_len(res: &Response)-> io::Result<u64> {
  match res.content_length() {
    Some(len)=> Ok(len),
    _=> Err(Error::new(ErrorKind::InvalidData,"couldn't retrieve the content-size"))
  }
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

