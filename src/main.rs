#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

use {
    rustysynth::{MidiFile, MidiFileSequencer, SoundFont, Synthesizer, SynthesizerSettings},
    std::{fs::File, io::Write, sync::Arc},
};

fn main() {
    let mut args = std::env::args_os().skip(1);
    let midi_path = args.next().expect("Need midi path");
    let sf_path = args.next().expect("Need soundfont path");
    let midi = MidiFile::new(&mut File::open(&midi_path).unwrap()).unwrap();
    let sf = SoundFont::new(&mut File::open(&sf_path).unwrap()).unwrap();
    let settings = SynthesizerSettings::new(44_100);
    let synth = Synthesizer::new(&Arc::new(sf), &settings).unwrap();
    let mut seq = MidiFileSequencer::new(synth);
    let buf_len = 65_536;
    let mut left: Vec<f32> = vec![0.; buf_len];
    let mut right: Vec<f32> = vec![0.; buf_len];
    let mut interleaved = vec![0.; buf_len * 2];
    let total_len = midi.get_length();
    seq.play(&Arc::new(midi), false);
    while !seq.end_of_sequence() {
        seq.render(&mut left, &mut right);
        let mut idx = 0;
        for (l, r) in left.iter().zip(right.iter()) {
            interleaved[idx] = *l;
            interleaved[idx + 1] = *r;
            idx += 2;
        }
        std::io::stdout()
            .lock()
            .write_all(bytemuck::cast_slice(&interleaved))
            .unwrap();
        eprint!("Rendering... ({}/{})\r", seq.get_position(), total_len);
    }
}
