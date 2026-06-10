#[cfg(feature = "bridge")]
use std::sync::Arc;
#[cfg(feature = "bridge")]
use std::time::Duration;
#[cfg(feature = "bridge")]
use omid::{OmidHostDispatcher, DispatcherStats, Midi1Translator};
#[cfg(feature = "bridge")]
use omid::queue::SpscRingBuffer;
#[cfg(feature = "bridge")]
use midir::{MidiInput, MidiOutput};

#[cfg(feature = "bridge")]
fn run_bridge() -> Result<(), Box<dyn std::error::Error>> {
    println!("Initializing OMID Legacy Bridge Daemon...");

    // Create dispatcher queues
    let stats = Arc::new(DispatcherStats::default());
    let q0 = Arc::new(SpscRingBuffer::new());
    let q1 = Arc::new(SpscRingBuffer::new());
    let rx_queues = vec![q0.clone(), q1.clone()];
    let tx_queue = Arc::new(SpscRingBuffer::new());

    // Spawn dispatcher
    let dispatcher = OmidHostDispatcher::new(2, rx_queues.clone(), tx_queue.clone(), stats.clone());
    let dispatcher = Arc::new(dispatcher);

    // Initialize MIDI input
    let midi_in = MidiInput::new("OMID-Bridge-In")?;
    let in_ports = midi_in.ports();
    let mut conn_in = None;

    if !in_ports.is_empty() {
        let port_name = midi_in.port_name(&in_ports[0])?;
        println!("Binding to MIDI Input: {}", port_name);

        let dispatcher_clone = Arc::clone(&dispatcher);
        let rx_queues_clone = rx_queues.clone();
        
        let connection = midi_in.connect(
            &in_ports[0],
            "omid-read-input",
            move |_timestamp, message, _| {
                if let Ok(packet) = Midi1Translator::to_omid(message) {
                    let _ = dispatcher_clone.dispatch(packet, &rx_queues_clone);
                }
            },
            (),
        )?;
        conn_in = Some(connection);
        println!("MIDI Input listener active.");
    }

    // Initialize MIDI output
    let midi_out = MidiOutput::new("OMID-Bridge-Out")?;
    let out_ports = midi_out.ports();
    let mut conn_out = None;

    if !out_ports.is_empty() {
        let port_name = midi_out.port_name(&out_ports[0])?;
        println!("Binding to MIDI Output: {}", port_name);
        let out_conn = midi_out.connect(&out_ports[0], "omid-write-output")?;
        conn_out = Some(out_conn);
        println!("MIDI Output writer active.");
    }

    // Run polling thread for outbound (VST to Hardware) feedback
    let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r_clone = Arc::clone(&running);
    let tx_queue_clone = tx_queue.clone();
    
    let poll_thread = std::thread::spawn(move || {
        while r_clone.load(std::sync::atomic::Ordering::Relaxed) {
            if let Some(packet) = tx_queue_clone.pop() {
                if let Some(ref mut out_conn) = conn_out {
                    let mut buf = [0u8; 3];
                    if Midi1Translator::to_midi1(packet, &mut buf).is_ok() {
                        let _ = out_conn.send(&buf);
                    }
                }
            } else {
                std::thread::sleep(Duration::from_millis(1));
            }
        }
    });

    println!("Bridge Daemon successfully running. Press [Enter] to terminate.");
    let mut input = String::new();
    let _ = std::io::stdin().read_line(&mut input);

    println!("Shutting down Bridge Daemon...");
    running.store(false, std::sync::atomic::Ordering::Relaxed);
    let _ = poll_thread.join();

    // Drop connections to free Arc references
    drop(conn_in);

    if let Ok(d) = Arc::try_unwrap(dispatcher) {
        d.shutdown();
    } else {
        println!("Warning: Could not cleanly shutdown dispatcher due to active references.");
    }

    Ok(())
}

fn main() {
    #[cfg(feature = "bridge")]
    {
        if let Err(err) = run_bridge() {
            eprintln!("Bridge Error: {}", err);
            std::process::exit(1);
        }
    }
    #[cfg(not(feature = "bridge"))]
    {
        println!("Please build with '--features bridge' to run the Omid legacy bridge daemon.");
    }
}
