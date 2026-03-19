use brook_http_worker::worker::Worker;
use brook_http_worker::job::{Job, JobAbstract};

// 1. Defina sua lógica
struct MyHttpJob;

impl JobAbstract for MyHttpJob {
    fn perform(&self, mut job: Job) {
        // Agora você acessa diretamente!
        println!("Processando Canal: {}", job.channel);
        println!("Payload recebido: {:?}", job.payload);

        // Exemplo: se o payload for um objeto e você quiser um campo específico
        // if let Some(url) = job.payload.get("url") {
        //     println!("URL para disparar: {}", url);
        // }

        // Resposta via Redis usando o canal que veio no JSON
        self.success_response(&mut job, "Processado com sucesso".to_string());
    }
}

// 2. O main fica exatamente como você queria
fn main() {
    let mut w = Worker::new();

    w.add_job("third-job", MyHttpJob);

    w.start();
}
