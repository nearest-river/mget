
use std::cmp::min;
use reqwest::Client;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::time::{
  self,
  Duration
};


#[tokio::test]
async fn progress() {
  let mut downloaded=0;
  let total_size=231231231;

  let pb=ProgressBar::new(total_size);
  pb.set_style(progress_style());

  while downloaded < total_size {
    let new=min(downloaded + 223211, total_size);
    downloaded=new;
    pb.set_position(new);
    time::sleep(Duration::from_millis(12)).await;
  }

  pb.finish_with_message("downloaded");
}


#[inline(always)]
fn progress_style()-> ProgressStyle {
  ProgressStyle::with_template(
    "[{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})"
  )
  .unwrap()
  .progress_chars("#>-")
}


async fn _get_content_size()-> reqwest::Result<u64> {
  static URL: &str="http://84.22.44.211/09.09/Interstellar%20(2014)%20(2014)%20[1080p]/Interstellar.2014.2014.1080p.BluRay.x264.YIFY.mp4";
  let client=Client::new();
  let res=client.head(URL)
  .send()
  .await?;

  Ok(match res.content_length() {
    Some(size)=> size,
    _=> f64::INFINITY as _
  })
}

fn _init_logger() {
  tracing_subscriber::FmtSubscriber::builder()
  .compact()
  .with_line_number(false)
  .without_time()
  .with_level(false)
  .with_target(false)
  .init();
}



