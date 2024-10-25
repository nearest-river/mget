
use futures::future;
use std::{
  env,
  path::PathBuf,
  sync::LazyLock
};

use tokio::{
  fs::OpenOptions,
  task::JoinHandle,
  io::AsyncWriteExt
};

use reqwest::{
  Url,
  Client,
  IntoUrl,
  Response
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
  tasks: Vec<JoinHandle<anyhow::Result<()>>>
}

impl Downloader {
  #[inline]
  pub fn new()-> Self {
    Self::default()
  }

  #[inline]
  pub fn add_to_queue(&mut self,url: Url) {
    self.tasks.push(tokio::spawn(download(url)))
  }

  #[inline]
  pub fn extent_queue<I: ExactSizeIterator<Item=Url>>(&mut self,iter: I) {
    self.tasks.reserve(iter.len());
    self.tasks.extend(
      iter.map(|url| tokio::spawn(download(url)))
    );
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


async fn download<T: IntoUrl>(url: T)-> anyhow::Result<()> {
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
  let mut file=OpenOptions::new()
  .create(true)
  .write(true)
  .read(true)
  .open(DOWNLOAD_DIR.join(file_name))
  .await?;

  file.write_all(&res.bytes().await?).await?;
  Ok(())
}

#[inline]
async fn get<T: IntoUrl>(url: T)-> reqwest::Result<Response> {
  static CLIENT: LazyLock<Client>=LazyLock::new(|| Client::new());

  CLIENT.get(url)
  .send()
  .await
}

