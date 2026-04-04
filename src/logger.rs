use std::path::PathBuf;
use std::sync::OnceLock;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};

pub struct Logger;
struct LoggerConfig {
    log_dir: PathBuf,
    process_name: String,
}

pub enum LoggerLevel {
    LOG_DEBUG,
    LOG_INFO,
    LOG_WARN,
    LOG_ERR
}

static CONFIG: OnceLock<LoggerConfig> = OnceLock::new();

pub fn init (log_path: &str, process_name: &str) {
    // ... create path of logs ...
    let path = PathBuf::from(log_path);
    if !path.exists() {
        std::fs::create_dir_all(&path).expect(format!("Error creating {} log directory:", log_path).as_str());
    }

    let config = LoggerConfig {
        log_dir: path,
        process_name: process_name.to_string(),
    };

    CONFIG.set(config).ok();
}

fn log_file () -> io::Result<File> {
    let config = CONFIG.get().expect("Logger não inicializado. Chame init_logger()");
    let date_str = chrono::Local::now().format("%Y-%m-%d").to_string();
    let file_name = format!("brook-{}-{}.log", config.process_name, date_str);
    let file_path = config.log_dir.join(file_name);

    // O_APPEND garante atomicidade na escrita entre diferentes processos
    // que possam estar a tentar escrever no mesmo ficheiro (se partilharem o nome)
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)

}

pub fn log (level: &str, msg: &str) {
    if let Ok(mut file) = log_file() {
        let now = chrono::Local::now();
        let pid = std::process::id();

        // Formatação da linha de log
        let line = format!(
            "[{}][{}][PID:{}]: {}\n",
            now.format("%Y-%m-%dT%H:%M:%S"),
            level,
            pid,
            msg
        );

        // Escrita atómica (até ao limite do buffer do SO)
        let _ = file.write_all(line.as_bytes());
        let _ = file.flush();
    }
}
