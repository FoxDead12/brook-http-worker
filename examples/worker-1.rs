use brook_http_worker::worker::Worker;
use brook_http_worker::job::{Job, JobAbstract};
use serde::{Serialize, Deserialize};

#[derive(Serialize)]
struct OLA {
    id: u16
}

#[derive(Deserialize, Debug)]
struct ThirdJobPayload { user_id: u32, name: String }

struct ThirdJob;
impl JobAbstract for ThirdJob {
    fn perform(&self, mut job: Job) {
        // Agora você acessa diretamente!
        println!("[third-job] Processando Canal: {}", job.channel);

        if let Some(raw_payload) = &job.payload {
            // Tenta converter o Value para a sua Struct específica
            match serde_json::from_value::<ThirdJobPayload>(raw_payload.clone()) {
                Ok(valid_payload) => {
                    // Aqui você tem o payload tipado e validado
                    println!("Payload matches schema: {:?}", valid_payload);
                }
                Err(e) => {
                    let error_msg = format!("Invalid payload schema for Job {}: {}", job.id, e);
                    self.success_response(&mut job, error_msg.as_str(), Some("Need send all payload"), None);
                }
            }
        }

        // Exemplo: se o payload for um objeto e você quiser um campo específico
        // if let Some(url) = job.payload.get("url") {
        //     println!("URL para disparar: {}", url);
        // }

        // Resposta via Redis usando o canal que veio no JSON
        let meu_dado = OLA { id: 10 };
        self.success_response(&mut job, "Processado com sucesso", None, Some(serde_json::json!(meu_dado)));
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

        self.success_response(&mut job, "Processado com sucesso", None, Some(serde_json::json!(meu_dado)));
    }
}

// 2. O main fica exatamente como você queria
fn main() {
    let mut w = Worker::new();

    w.add_job("third-job", ThirdJob);
    w.add_job("firt-job", FirstJob);

    w.start();
}
