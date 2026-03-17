mod job {
  /**
   * Interface to job struct all jobs will use this methods
   */
  pub trait JobHandler: Send + Sync {
    fn handle(&self);
  }
}
