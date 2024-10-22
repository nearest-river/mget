
use reqwest::IntoUrl;
use tokio::{
  fs::OpenOptions,
  io::AsyncWriteExt
};

use std::{
  env,
  path::PathBuf,
  sync::LazyLock
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

pub async fn download<T: IntoUrl>(url: T)-> anyhow::Result<()> {
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

  let res=reqwest::get(url).await?;
  let mut file=OpenOptions::new()
  .create(true)
  .write(true)
  .read(true)
  .open(DOWNLOAD_DIR.join(file_name))
  .await?;

  file.write_all(&res.bytes().await?).await?;
  Ok(())
}


