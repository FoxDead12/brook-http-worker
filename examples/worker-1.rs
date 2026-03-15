use brook_http_worker::{RustWorker, JobHandler};


/**
 * Job implementation can be a new file
 */

struct ThirdJob;

impl JobHandler for ThirdJob {
  fn handle (&self) {
    println!("HEIIIIII ENTREI NO MEU JOB");
  }
}



// ... worker examle can be another file ...
fn main () {
  let mut worker = RustWorker::new();
  worker.register_job("third-job", ThirdJob);
  worker.start();
}
