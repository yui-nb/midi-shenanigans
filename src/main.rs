use std::convert::TryFrom;
use std::error::Error;
use std::io::{stdin, stdout, Write};

use clap::{App, SubCommand};
use midir::{Ignore, MidiIO, MidiInput, MidiOutput};
use wmidi::{FromBytesError, MidiMessage};

fn main() {
    let matches = App::new("MIDI Shenanigans built around the KORG NTS-1 digital Synthesizer")
        .version("0.0.1")
        .author("Yui E. <yui.github@mailbox.org>")
        .about("Doing stuff with MIDI")
        .subcommand(SubCommand::with_name("listen").about("Listen to the input of a MIDI Device"))
        .subcommand(SubCommand::with_name("forward").about("Forward MIDI Ports"))
        .get_matches();
    match matches.subcommand_name() {
        Some("forward") => forward_port().unwrap_or_else(|e| println!("Error: {}", e)),
        Some("listen") => listen_to_port().unwrap_or_else(|e| println!("Error: {}", e)),
        _ => println!("No command supplied. Exiting."),
    }
}

fn print_midi_message(midi_message: &[u8]) -> Result<(), FromBytesError> {
    use MidiMessage::*;
    let message = wmidi::MidiMessage::try_from(midi_message)?;
    match message {
        NoteOff(_, _, _) => {
            println!("Note off:");
        }
        NoteOn(channel, note, velocity) => {
            println!("Note on:");
            println!(
                "Channel: {:?} | Note: {:?} | Velocity: {}",
                channel,
                note,
                u8::from(velocity)
            );
        }
        PitchBendChange(channel, pitch_bend) => {
            println!("Pitch Bend Change:");
            println!("Channel: {:?} | Value: {}", channel, u16::from(pitch_bend));
        }
        ControlChange(channel, function, value) => {
            // TODO:: Implement Function to decode ControlFunction and to Map ControlFunctions
            // Maybe read Mapping form text file
            println!("Control Change:");
            println!(
                "Channel: {:?} | Function: {} | Value: {}",
                channel,
                u8::from(function),
                u8::from(value)
            );
        }
        SysEx(message) => {
            println!("System Exclusive Message: {:?}", message);
        }
        TimingClock => {
            println!("CLOCK");
        }
        _ => println!("Placeholder..."),
    }
    Ok(())
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
    let _conn_in = midi_in.connect(
        &inp_port,
        &inp_port_name,
        move |_, message, _| {
            conn_out
                .send(message)
                .unwrap_or_else(|_| println!("Error sending this message"));
        },
        (),
    );
    println!("Press enter to exit...");
    let mut inp_buffer = String::new();
    stdin().read_line(&mut inp_buffer)?;
    Ok(())
}

fn listen_to_port() -> Result<(), Box<dyn Error>> {
    let mut midi_in = MidiInput::new("MIDI Input")?;
    midi_in.ignore(get_ignore()?);
    let inp_port = select_port(&midi_in, "input")?;
    let inp_port_name = midi_in.port_name(&inp_port)?;
    println!("\nConnecting....");
    let _conn_in = midi_in.connect(
        &inp_port,
        &inp_port_name,
        move |_stamp, message, _| {
            //println!("{}: {:?} (len = {})", stamp, message, message.len());
            match print_midi_message(&message) {
                Ok(_) => {}
                Err(e) => {
                    println!("Error parsing MIDI Message: {}", e);
                }
            }
        },
        (),
    )?;
    let mut inp_buffer = String::new();
    stdin().read_line(&mut inp_buffer)?;
    println!("Closing connection....");
    Ok(())
}

fn get_ignore() -> Result<Ignore, Box<dyn Error>> {
    println!("Which Messages do you wan't to ignore?");
    println!(
        "1: None, 2: SysEx, 3: Time, 4: SysEx and Time, 5: ActiveSense, 6: SysEx and ActiveSense
7: TimeAndActiveSense, 8: All"
    );
    let mut inp_buffer = String::new();
    stdin().read_line(&mut inp_buffer)?;
    match inp_buffer.trim().parse::<usize>() {
        Ok(val) => match val {
            1 => Ok(Ignore::None),
            2 => Ok(Ignore::Sysex),
            3 => Ok(Ignore::Time),
            4 => Ok(Ignore::SysexAndTime),
            5 => Ok(Ignore::ActiveSense),
            6 => Ok(Ignore::SysexAndActiveSense),
            7 => Ok(Ignore::TimeAndActiveSense),
            8 => Ok(Ignore::All),
            _ => {
                println!("Input out of range. Choosing None as default");
                Ok(Ignore::None)
            }
        },
        Err(_) => {
            println!("Could not parse Input. Chosing None as default!");
            Ok(Ignore::None)
        }
    }
    //Ok(Ignore::None)
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
