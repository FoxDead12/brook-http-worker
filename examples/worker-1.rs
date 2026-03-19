use brook_http_worker::worker::Worker;
use brook_http_worker::job::{JobAbstract, Job};

struct MyHttpJob;

impl JobAbstract for MyHttpJob {

  fn perform(&self, _job: Job) {

    println!("Job a fazer algo!");

  }
}

// ... worker examle can be another file ...
fn main () {
  let mut w= Worker::new();
  w.add_job("third-job", MyHttpJob);
  w.start();
}
