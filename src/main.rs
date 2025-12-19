#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

use {
    rustysynth::{MidiFile, MidiFileSequencer, SoundFont, Synthesizer, SynthesizerSettings},
    std::{fs::File, io::Write, path::Path, sync::Arc},
};

// Copied from mpvfrog
pub struct FfmpegTimeFmt(pub f64);

impl std::fmt::Display for FfmpegTimeFmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let secs = self.0;
        let hh = secs / 3600.0;
        let mm = hh.fract() * 60.0;
        let ss = mm.fract() * 60.0;
        write!(
            f,
            "{:02.0}:{:02.0}:{:02.0}.{:03}",
            hh.floor(),
            mm.floor(),
            ss.floor(),
            (ss.fract() * 1000.0).round() as u64
        )
    }
}

fn main() {
    let mut args = std::env::args_os().skip(1);
    let midi_path = args.next().expect("Need midi path");
    let sf_path = args.next().expect("Need soundfont path");
    let midi = MidiFile::new(&mut File::open(&midi_path).unwrap()).unwrap();
    let sf = SoundFont::new(&mut File::open(&sf_path).unwrap()).unwrap();
    let settings = SynthesizerSettings::new(44_100);
    let synth = Synthesizer::new(&Arc::new(sf), &settings).unwrap();
    let mut seq = MidiFileSequencer::new(synth);
    let buf_len = 16_384;
    let mut left: Vec<f32> = vec![0.; buf_len];
    let mut right: Vec<f32> = vec![0.; buf_len];
    let mut interleaved = vec![0.; buf_len * 2];
    let total_len = midi.get_length();
    seq.play(&Arc::new(midi), false);
    let file_name = Path::new(&midi_path).file_name().unwrap_or("anon".as_ref());
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
        eprint!(
            "[{}] {}/{}\r",
            file_name.display(),
            FfmpegTimeFmt(seq.get_position()),
            FfmpegTimeFmt(total_len)
        );
    }
}
