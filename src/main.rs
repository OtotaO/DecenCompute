use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use rsa::{PublicKey, RsaPrivateKey, PaddingScheme, pkcs8::FromPrivateKey};
use log::{info, error};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::time::Duration;

// Configuration
const HOST: &str = "127.0.0.1";
const PORT: u16 = 5000;
const BUFFER_SIZE: usize = 4096;
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(10);

// Define types for clarity
type ComputationResult = Vec<f64>;
type ComputationFunction = fn(Vec<f64>) -> ComputationResult;

// Define message types for communication
#[derive(Serialize, Deserialize)]
struct Task {
    data: Vec<f64>,
    computation: ComputationFunction,
}

#[derive(Serialize, Deserialize)]
struct Response {
    result: ComputationResult,
}

// Function to receive data from the client
fn receive_data(mut stream: &TcpStream, private_key: &RsaPrivateKey) -> Option<Task> {
    let mut buffer = Vec::new();
    if let Err(err) = stream.read_to_end(&mut buffer) {
        error!("Failed to read data from stream: {}", err);
        return None;
    }

    let mut decoder = ZlibDecoder::new(&buffer[..]);
    let mut decoded_data = Vec::new();
    if let Err(err) = decoder.read_to_end(&mut decoded_data) {
        error!("Failed to decode data: {}", err);
        return None;
    }

    let padding = PaddingScheme::new_pkcs1v15_encrypt();
    let decrypted_data = match private_key.decrypt(padding, &decoded_data) {
        Ok(data) => data,
        Err(err) => {
            error!("Failed to decrypt data: {}", err);
            return None;
        }
    };

    let mut deserializer = Deserializer::new(&decrypted_data[..]);
    match Deserialize::deserialize(&mut deserializer) {
        Ok(task) => Some(task),
        Err(err) => {
            error!("Failed to deserialize task: {}", err);
            None
        }
    }
}

// Function to send data to the client
fn send_data(mut stream: &TcpStream, data: &Response, public_key: &PublicKey) -> Result<(), Box<dyn std::error::Error>> {
    let mut serializer = Serializer::new(Vec::new());
    data.serialize(&mut serializer)?;
    let serialized_data = serializer.into_inner();

    let padding = PaddingScheme::new_pkcs1v15_encrypt();
    let encrypted_data = public_key.encrypt(&mut rand::thread_rng(), padding, &serialized_data)?;

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&encrypted_data)?;
    let compressed_data = encoder.finish()?;

    if let Err(err) = stream.write_all(&compressed_data) {
        return Err(Box::new(err));
    }

    Ok(())
}

// Function to handle each client connection
fn handle_client(mut stream: TcpStream, private_key: Arc<RsaPrivateKey>, public_key: Arc<PublicKey>, active_connections: Arc<Mutex<HashMap<String, bool>>>, shutdown_signal: Arc<AtomicBool>) {
    let addr = stream.peer_addr().unwrap();
    info!("[NEW CONNECTION] {} connected.", addr);

    // Set up heartbeat mechanism
    let (heartbeat_tx, heartbeat_rx) = mpsc::channel();
    let heartbeat_thread = thread::spawn(move || {
        loop {
            thread::sleep(HEARTBEAT_INTERVAL);
            if heartbeat_tx.send(()).is_err() {
                break;
            }
        }
    });

    loop {
        // Check if a shutdown signal has been received
        if shutdown_signal.load(Ordering::Relaxed) {
            break;
        }

        // Check for heartbeat
        if let Ok(()) = heartbeat_rx.try_recv() {
            // Heartbeat received, continue processing
        } else {
            // Heartbeat missed, assume client disconnected
            info!("[DISCONNECT] {} disconnected due to missed heartbeat.", addr);
            break;
        }

        match receive_data(&stream, &private_key) {
            Some(task) => {
                let result = (task.computation)(task.data);
                let response = Response { result };
                if let Err(e) = send_data(&stream, &response, &public_key) {
                    error!("[COMPUTATION ERROR] Error occurred during computation for {}: {}", addr, e);
                    break;
                }
            }
            None => {
                info!("[DISCONNECT] {} disconnected.", addr);
                break;
            }
        }
    }

    // Clean up heartbeat thread
    drop(heartbeat_tx);
    heartbeat_thread.join().unwrap();

    let mut connections = active_connections.lock().unwrap();
    connections.remove(&addr.to_string());
    drop(connections);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    // Load private key (TODO)
    let private_key = Arc::new(/* Load private key */);
    // Load public key (TODO)
    let public_key = Arc::new(/* Load public key */);

    // Set up TCP listener
    let listener = TcpListener::bind((HOST, PORT))?;
    let active_connections: Arc<Mutex<HashMap<String, bool>>> = Arc::new(Mutex::new(HashMap::new()));

    // Set up shutdown mechanism
    let shutdown_signal = Arc::new(AtomicBool::new(false));
    let (shutdown_tx, shutdown_rx) = mpsc::channel();

    // Spawn a thread to handle shutdown
    let shutdown_tx_clone = shutdown_tx.clone();
    thread::spawn(move || {
        // Wait for user input to shut down the server
        let _ = std::io::stdin().read_line(&mut String::new());
        shutdown_tx_clone.send(()).unwrap();
    });

    // Accept and handle incoming connections
    for stream in listener.incoming() {
        // Check for shutdown signal
        if shutdown_signal.load(Ordering::Relaxed) {
            break;
        }

        let private_key = private_key.clone();
        let public_key = public_key.clone();
        let active_connections = active_connections.clone();
        let shutdown_signal = shutdown_signal.clone();
        let shutdown_tx = shutdown_tx.clone();

        thread::spawn(move || {
            handle_client(stream.unwrap(), private_key, public_key, active_connections, shutdown_signal);
            if let Ok(()) = shutdown_rx.try_recv() {
                // Shutdown signal received, send signal to other threads
                shutdown_signal.store(true, Ordering::Relaxed);
                shutdown_tx.send(()).unwrap();
            }
        });
    }

    // Graceful shutdown
    info!("Shutting down server...");
    // Additional cleanup if needed

    Ok(())
}
