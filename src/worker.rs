pub mod job {
    use beanstalkc::Beanstalkc;
    use redis::{Commands, Connection};
    use serde::Serialize;

    // Contexto que será injetado no seu job
    pub struct Job<'a> {
        pub id: u64,
        pub channel: String,
        pub payload: Option<serde_json::Value>,
        pub beanstalkd: &'a mut Beanstalkc,
        pub redis: &'a mut Connection,
    }

    pub trait JobAbstract {
        fn perform(&self, job: Job);

        fn success_response (&self, job: &mut Job, message: &str, detail: Option<&str>, data: Option<serde_json::Value>) {
            let mut payload_obj = serde_json::json!({
                "message": message,
                "code": 200
            });

            if let Some(extra) = data {
                payload_obj["data"] = extra;
            }

            if let Some(extra) = detail {
                payload_obj["detail"] = serde_json::Value::String(extra.to_string());
            }

            let response = JobResponse {
                job_id: job.id,
                status: 200,
                headers: serde_json::json!({}),
                payload: payload_obj
            };

            match serde_json::to_string(&response) {
                Ok(json_message) => {
                    let channel = job.channel.clone();
                    let _: Result<i32, _> = job.redis.publish(channel, json_message);
                }
                Err(e) => eprintln!("Erro ao serializar resposta: {}", e),
            }

        }
    }

    #[derive(Serialize)]
    struct JobResponse { job_id: u64, status: u16, headers: serde_json::Value, payload: serde_json::Value }
}

pub mod worker {
    use crate::worker::job::{Job, JobAbstract};
    use beanstalkc::Beanstalkc;
    use serde::Deserialize;
    use std::{collections::HashMap, env, fs, thread, time::Duration};

    pub struct Worker {
        beanstalkd: Beanstalkc,
        redis: redis::Connection,
        jobs: HashMap<String, Box<dyn JobAbstract>>,
    }

    impl Worker {
        pub fn new() -> Self {
            let config = Self::load_config();

            let beanstalkd = Beanstalkc::new()
                .host(&config.beanstalkd.host)
                .port(config.beanstalkd.port)
                .connect()
                .expect("Failed to establish Beanstlakd connection: Service might be unreachable or credentials are incorrect");

            let redis_url = format!("redis://{}:{}/", config.redis.host, config.redis.port);
            let redis = redis::Client::open(redis_url)
                .expect("Invalid Redis connection URL: Please check the connection string format")
                .get_connection()
                .expect("Failed to establish Redis connection: Service might be unreachable or credentials are incorrect");

            Worker { beanstalkd, redis, jobs: HashMap::new() }
        }

        fn load_config() -> WorkerConfig {
            let args: Vec<String> = env::args().collect();
            let path = args.get(2).expect("Usage: cargo run -- -config <config_file.yml>");
            let content = fs::read_to_string(path).expect("Failed to read configuration file: Check if the path is correct and permissions are set");
            serde_yaml::from_str(&content).expect("Invalid configuration format: Failed to parse YAML content")
        }

        pub fn add_job<T: JobAbstract + 'static>(&mut self, tube: &str, job: T) {
            self.beanstalkd.watch(tube).expect("Failed to watch tube: Connection lost or invalid tube name provided");
            self.jobs.insert(tube.to_string(), Box::new(job));
        }

        pub fn start(&mut self) {
            println!("Worker iniciado. Aguardando jobs...");
            loop {
                if let Ok(bean_job) = self.beanstalkd.reserve() {
                    let id = bean_job.id();
                    let parsed: Result<BeanstalkPayload, _> = serde_json::from_slice(bean_job.body());
                    match parsed {
                        Ok(data) => {
                            if let Ok(stats) = self.beanstalkd.stats_job(id) {
                                let tube = stats.get("tube").map(|s| s.as_str()).unwrap_or("");

                                if let Some(handler) = self.jobs.get(tube) {
                                    let context = Job {
                                        id,
                                        channel: data.channel,
                                        payload: data.payload,
                                        beanstalkd: &mut self.beanstalkd,
                                        redis: &mut self.redis,
                                    };
                                    handler.perform(context);
                                }
                            }
                            let _ = self.beanstalkd.delete(id);
                        },
                        Err(e) => {
                            eprintln!("Failed to parse JSON payload for Job {}: {}", id, e);
                            let _ = self.beanstalkd.delete(id);
                        }
                    }

                    let _ = self.beanstalkd.delete(id);
                }
            }
        }
    }

    #[derive(Deserialize)] struct WorkerConfig { beanstalkd: BeanstalkdConfig, redis: RedisConfig }
    #[derive(Deserialize)] struct BeanstalkdConfig { host: String, port: u16 }
    #[derive(Deserialize)] struct RedisConfig { host: String, port: u16 }
    #[derive(Deserialize, Debug)] struct BeanstalkPayload { channel: String, payload: Option<serde_json::Value> }
}
