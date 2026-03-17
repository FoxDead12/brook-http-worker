use std::collections::HashMap;
use std::{env, fs};
use std::process::exit;
use beanstalkc::{Beanstalkc};
use serde::Deserialize;
use job;

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
struct RustWorkerConfig {
  beanstalkd: BeanstalkdConfig,
  redis: RedisConfig,
}

pub struct RustWorker {
  _config: RustWorkerConfig,
  _beanstalkd: Beanstalkc,
  _redis: redis::Connection,
  _handlers: HashMap<String, Box<dyn job::JobHandler>>,
}

impl RustWorker {

  /**
   * Worker start here, will handle job and execute custom logic
   */
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

    RustWorker {
      _config,
      _beanstalkd,
      _redis,
      _handlers: HashMap::new(),
    }
  }

  /**
   * Add jobs logics
   */
  pub fn register_job<H: job::JobHandler + 'static>(&mut self, tube: &str, handler: H) {
    // Fazemos o watch do tubo automaticamente ao registar
    self._beanstalkd.watch(tube).expect("Can't watch tube");
    self._handlers.insert(tube.to_string(), Box::new(handler));
  }

  /**
   * Load configuration from .yml file
   *
   */
  fn _configure () -> RustWorkerConfig {
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
    let _c: RustWorkerConfig = serde_yaml::from_str(&_yml_string).unwrap_or_else(|e| {
      eprintln!("Error: Invalid YAML format: {}", e);
      exit(1);
    });

    _c
  }

  /**
   * Function to make worker start
   */
  pub fn start (&mut self) {
    // ... worker start here ...
    loop {
      match self._beanstalkd.reserve() {
        Ok(job) => {
          let job_id = job.id();
          let job_body = job.body().to_vec();
          match self._beanstalkd.stats_job(job_id) {
            Ok(stats) => self.received_job(job_id, job_body, stats),
            Err(e) => {
              eprintln!("Falha ao buscar stats to job: {:?}", e);
            }
          }
        },
        Err(e) => {
          eprintln!("Falha ao reservar job (Timeout ou Erro): {:?}", e);
          std::thread::sleep(std::time::Duration::from_millis(500));
        }
      };
    }
  }

  fn received_job (&mut self, job_id: u64, job_body: Vec<u8>, job_stats: HashMap<String, String>) {

    // ... get tube from job ...
    let tube = match job_stats.get("tube") {
      Some(tube) => tube,
      None => ""
    };

    eprintln!("Job recebido do tubo {:?} com id {} e o seguinte body: {:?}", tube, job_id, job_body);

    // ... transform tube name to create class ...
    if let Some(handler) = self._handlers.get(tube) {
      handler.handle();
    }

  }

}

//
// fn main() {
//
//   let mut _beanstalkd = Beanstalkc::new()
//     .host("127.0.0.1")
//     .port(11301)
//     .connect()
//     .expect("Can't create beanstalkd client");
//
//   // ... watch all jobs from config file ...
//   _beanstalkd.watch("third-job").expect("Can't watch jobs");
//
//   loop {
//     // ... get job from tubes
//     if let Ok(job) = _beanstalkd.reserve() {
//
//       println!("Processando job {:?} ({} bytes)", std::str::from_utf8(job.body()), job.body().len());
//     }
//   }
// }
