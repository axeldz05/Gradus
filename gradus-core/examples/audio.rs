use gradus_core::note_detector::NoteDetector;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    println!("Iniciando prueba de micrófono...");
    
    let (tx, rx) = mpsc::channel();

    let _stream = NoteDetector::connect(tx);
    
    println!("Escuchando... (Habla o toca una nota)");

    let timeout = Duration::from_secs(30);
    let start = std::time::Instant::now();

    loop {
        if let Ok(nota) = rx.try_recv() {
            println!(">>> Recibido: Nota={}, Freq={:.1}", nota.note, nota.frequency);
        }

        if start.elapsed() > timeout {
            println!("Prueba finalizada.");
            break;
        }
        
        thread::sleep(Duration::from_millis(10));
    }
}
