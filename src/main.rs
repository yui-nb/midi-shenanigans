use std::error::Error;
use std::io::{stdin, stdout, Write};

use midir::{Ignore, MidiIO, MidiInput, MidiOutput};

fn main() {
    println!("MIDI Shenanigans Version: 0.0.1\n");
    match forward_port() {
        Ok(_) => println!("All good! Goodbye..."),
        Err(e) => println!("DAVE COURTNEY'S ILLEGAL ERROR: {}", e),
    }
}

fn forward_port() -> Result<(), Box<dyn Error>> {
    let mut midi_in = MidiInput::new("MIDI Input")?;
    let midi_out = MidiOutput::new("MIDI Output")?;
    midi_in.ignore(Ignore::None);
    let inp_port = select_port(&midi_in, "input")?;
    let outp_port = select_port(&midi_out, "output")?;
    let inp_port_name = midi_in.port_name(&inp_port)?;
    let outp_port_name = midi_out.port_name(&outp_port)?;
    println!("\nForwarding ports....");
    let mut conn_out = midi_out.connect(&outp_port, &outp_port_name)?;
    let _conn_in = midi_in.connect(&inp_port, &inp_port_name, move |_, message, _ | {
        conn_out.send(message).unwrap_or_else(|_| { println!("WTF") });
    }, ());
    println!("Press enter to exit...");
    let mut inp_buffer = String::new();
    stdin().read_line(&mut inp_buffer)?;
    Ok(())
}

fn listen_to_port() -> Result<(), Box<dyn Error>> {
    let mut midi_in = MidiInput::new("MIDI Input")?;
    midi_in.ignore(Ignore::None);
    let inp_port = select_port(&midi_in, "input")?;
    let inp_port_name = midi_in.port_name(&inp_port)?;
    println!("\nConnecting....");
    let _conn_in = midi_in.connect(
        &inp_port,
        &inp_port_name,
        move |stamp, message, _| {
            println!("{}: {:?} (len = {})", stamp, message, message.len());
        },
        (),
    )?;
    let mut inp_buffer = String::new();
    stdin().read_line(&mut inp_buffer)?;
    println!("Closing connection....");
    Ok(())
}

fn select_port<T: MidiIO>(midi_io: &T, descr: &str) -> Result<T::Port, Box<dyn Error>> {
    println!("available {} ports:", descr);
    let ports = midi_io.ports();
    for (index, port) in ports.iter().enumerate() {
        println!("{}: {}", index, midi_io.port_name(port)?);
    }
    println!("Please select {} Port", descr);
    stdout().flush()?;
    let mut inp_buffer = String::new();
    stdin().read_line(&mut inp_buffer)?;
    let port = ports
        .get(inp_buffer.trim().parse::<usize>()?)
        .ok_or("Invalid port number")?;
    Ok(port.clone())
}
