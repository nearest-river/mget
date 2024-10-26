
const url=new URL("../target/tmp/Vinland.Saga.S01E01.1080p.Dubble.Farsi.RaycaMovie.com.mp4",import.meta.url);
const vid=await fetch(url);


while(true) {
  try {
    await Deno.serve(()=> vid).finished;
  } catch(_) {
    continue;
  }
}

