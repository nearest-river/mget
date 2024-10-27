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
  sync::RwLock,
  fs::OpenOptions,
  task::JoinHandle,
  io::AsyncWriteExt
};


#[derive(Default)]
pub struct Downloader {
  progress_bars: MultiProgress,
  tasks: RwLock<Vec<JoinHandle<anyhow::Result<()>>>>
}

macro_rules! download_task {
  ($this:expr,$url:expr,$path:expr)=> {
    tokio::spawn(download($this.progress_bars.clone(),$url,$path))
  };
}

impl Downloader {
  #[inline]
  pub fn new()-> Self {
    Self::default()
  }

  #[inline]
  pub async fn add_to_queue(&self,url: Url,path: PathBuf) {
    let mut tasks=self.tasks.write().await;
    tasks.push(download_task!(self,url,path))
  }

  #[inline]
  pub async fn extent_queue(&self,iter: impl ExactSizeIterator<Item=(Url,PathBuf)>) {
    let mut tasks=self.tasks.write().await;
    tasks.reserve(iter.len());
    tasks.extend(
      iter
      .map(|(url,path)| download_task!(self,url,path))
    );
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


async fn download(progress_bars: MultiProgress,url: Url,path: PathBuf)-> anyhow::Result<()> {
  let mut res=get(url).await?;
  let bar=progress_bars.add(
    ProgressBar::new(
      res.content_length()
      .unwrap_or(0)
    ).with_style(progress_style())
  );

  let mut file=OpenOptions::new()
  .create(true)
  .write(true)
  .read(true)
  .open(path)
  .await?;

  while let Some(buf)=res.chunk().await? {
    file.write_all(&buf).await?;
    bar.inc(buf.len() as _);
  }

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

