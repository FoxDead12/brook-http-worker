pub mod job {
  use beanstalkc::Beanstalkc;

  pub struct Job {
    _beanstalkd: Beanstalkc,
    _redis: redis::Connection
  }

  pub trait JobAbstract  {
    fn setup(&self, _job: Job);
    fn perform(&self) {
      println!("Method perform need be defined in struct impl");
    }
  }
}

pub mod worker {
  use beanstalkc::Beanstalkc;
  use serde::Deserialize;
  use std::collections::HashMap;
  use std::env;
  use std::fs;
  use std::process::exit;
  use crate::job::JobAbstract;

  pub struct Worker {
    _config: WorkerConfig,
    _beanstalkd: Beanstalkc,
    _redis: redis::Connection,
    _jobs: HashMap<String, Box<dyn JobAbstract>>,
  }

  impl Worker {
    pub fn new () -> Self {
      // ... load configuration ...
      let mut _config = Self::_configure();

      let mut _beanstalkd = Beanstalkc::new()
      .host(&_config.beanstalkd.host)
      .port(_config.beanstalkd.port)
      .connect()
      .expect("Can't create beanstalkd client");

      let _redis_url = format!("redis://{}:{}/", _config.redis.host, _config.redis.port);
      let _redis = redis::Client::open(_redis_url)
        .expect("Invalid redis URL")
        .get_connection()
        .expect("Can't create redis client");

      Worker { _config, _beanstalkd, _redis, _jobs: HashMap::new() }
    }

    /**
     * Load configuration from .yml file
     *
     */
    fn _configure () -> WorkerConfig {
      // ... get args from script command ...
      let args: Vec<String> = env::args().collect();

      // ... parse param of command ...
      let _prop = args.get(1).map(|s| s.as_str()).unwrap_or("");
      if _prop != "-config" {
        eprintln!("Usage: bin/worker -config <file_path.yml>");
        exit(1);
      }

      // ... get path of file ...
      let _config_path = match args.get(2) {
        Some(val) => val.clone(),
        None => {
          eprintln!("Error: Missing file path for -config");
          eprintln!("Usage: bin/worker -config <file_path.yml>");
          exit(1);
        }
      };

      // ... parse yml file ...
      let _yml_string = fs::read_to_string(&_config_path).unwrap_or_else(|_| {
        eprintln!("Error: Could not read config file at {}", _config_path);
        exit(1);
      });

      // ... convert to lib struct ...
      let _c: WorkerConfig = serde_yaml::from_str(&_yml_string).unwrap_or_else(|e| {
        eprintln!("Error: Invalid YAML format: {}", e);
        exit(1);
      });

      _c
    }

    /**
     * Method to add tube and job to worker
     */
    pub fn add_job<T: JobAbstract + 'static> (&mut self, _tube: &str, _job: T) {
      self._beanstalkd.watch(_tube).expect("Can't watch tube");
      self._jobs.insert(_tube.to_string(), Box::new(_job));
    }

    /**
     * Start event loop to start reserve jobs
     */
    pub fn start (&mut self) {

      loop {
        // ... reserve job ...
        match self._beanstalkd.reserve() {
          Ok(job) => {
            // ... job has been reserve ...
            let job_id = job.id();
            let job_body = job.body().to_vec();
            match self._beanstalkd.stats_job(job_id) {
              Ok(stats) => {
                // ... get tube of job ...
                let tube = match stats.get("tube") {
                  Some(tube) => tube,
                  None => ""
                };

                eprintln!("Job recebido do tubo {:?} com id {} e o seguinte body: {:?}", tube, job_id, job_body);
                if let Some(_job) = self._jobs.get(tube) {
                  _job.perform();
                }

              },
              Err(e) => {
                eprintln!("Falha ao ver o estado do job: {:?}", e);
              }
            }
          },
          Err(e) => {
            // ... error reserving job ...
            eprintln!("Falha ao reservar job (Timeout ou Erro): {:?}", e);
            std::thread::sleep(std::time::Duration::from_millis(500));
          }
        }
      }
    }

  }

  #[derive(Deserialize, Debug)]
  struct BeanstalkdConfig {
    host: String,
    port: u16,
  }

  #[derive(Deserialize, Debug)]
  struct RedisConfig {
    host: String,
    port: u16,
  }

  #[derive(Deserialize, Debug)]
  struct WorkerConfig {
    beanstalkd: BeanstalkdConfig,
    redis: RedisConfig,
  }
}
