use brook_http_worker::worker::Worker;
use brook_http_worker::job::{Job, JobAbstract};
use serde::Serialize;

#[derive(Serialize)]
struct OLA {
    id: u16
}

struct ThirdJob;
impl JobAbstract for ThirdJob {
    fn perform(&self, mut job: Job) {
        // Agora você acessa diretamente!
        println!("[third-job] Processando Canal: {}", job.channel);
        println!("[third-job] Payload recebido: {:?}", job.payload);

        // Exemplo: se o payload for um objeto e você quiser um campo específico
        // if let Some(url) = job.payload.get("url") {
        //     println!("URL para disparar: {}", url);
        // }

        // Resposta via Redis usando o canal que veio no JSON
        let meu_dado = OLA { id: 10 };

        self.success_response(&mut job, "Processado com sucesso", Some(serde_json::json!(meu_dado)));
    }
}

struct FirstJob;
impl JobAbstract for FirstJob {
    fn perform(&self, mut job: Job) {
        // Agora você acessa diretamente!
        println!("[firt-job] Processando Canal: {}", job.channel);
        println!("[firt-job] Payload recebido: {:?}", job.payload);

        // Exemplo: se o payload for um objeto e você quiser um campo específico
        // if let Some(url) = job.payload.get("url") {
        //     println!("URL para disparar: {}", url);
        // }

        // Resposta via Redis usando o canal que veio no JSON
        let meu_dado = OLA { id: 10 };

        self.success_response(&mut job, "Processado com sucesso", Some(serde_json::json!(meu_dado)));
    }
}

// 2. O main fica exatamente como você queria
fn main() {
    let mut w = Worker::new();

    w.add_job("third-job", ThirdJob);
    w.add_job("firt-job", FirstJob);

    w.start();
}
