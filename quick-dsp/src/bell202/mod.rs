mod receiver;
mod sender;

use crate::filter::*;
pub use receiver::*;
pub use sender::*;

pub const BELL202_RATE: u32 = 1200;
pub const BELL202_MARK: u32 = 1200;
pub const BELL202_SPACE: u32 = 2200;
pub const BELL202_OPTIMAL_SAMPLE_RATE: u32 = 7200;

/// Bell 202 decoder.
///
/// Feed in samples into the returned filter and it will
/// occasionally spit out a frame. Does not check CRC.
///
/// Ideal sample rate is 7200. Maximum usable sample rate
/// is around 10000. If your sample rate is too high, you will need
/// to downsample first.
pub fn bell_202_decoder(sample_rate: u32) -> impl OneToOne<f32, Output = Option<Vec<u8>>> {
    #[cfg(not(test))]
    assert!(
        sample_rate <= 14000,
        "max sample rate:14000, given: {}",
        sample_rate
    );

    let space = (BELL202_SPACE as f32) / (sample_rate as f32);
    let mark = (BELL202_MARK as f32) / (sample_rate as f32);

    Discriminator::<f32, ()>::digital_default()
        .chain(FskDemod::new(space, mark))
        .chain(BitSampler::new(sample_rate, BELL202_RATE))
        .chain(NrziDecode::new().optional())
        .chain(HdlcDecode::default())
        // .inspect(|x| {
        //     if let Some(x) = x {
        //         println!("{:?}", x);
        //     }
        // })
        .chain(FrameCollector::default())
}

/// Bell 202 encoder.
///
/// Encodes a single frame of octets. Does not add CRC.
/// Input is an iterator of octets. Output is an iterator
/// samples at the given sample rate, with a preamble.
pub fn bell_202_encode<'a, Out, InIterator: Iterator<Item = u8> + 'a>(
    iter: InIterator,
    sample_rate: u32,
    amplitude: f32,
) -> impl Iterator<Item = <Decimator<f32, Out> as OneToOne<f32>>::Output> + 'a
where
    Decimator<f32, Out>: Default + OneToOne<f32>,
    Out: 'a,
{
    let samples_per_bit = (sample_rate as f32) / (BELL202_RATE as f32);
    let mark_freq = (BELL202_MARK as f32) / (sample_rate as f32);
    let space_freq = (BELL202_SPACE as f32) / (sample_rate as f32);

    iter.bits_lsb()
        .hdlc_encode()
        .nrzi_encode()
        .resample_nn(samples_per_bit)
        .map(move |x| match x {
            true => mark_freq,
            false => space_freq,
        })
        .apply_one_to_one(FmMod::new(amplitude))
        .apply_one_to_one(Decimator::<f32, Out>::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filter::Downsampler;

    #[test]
    fn test_bell_202_encode_decode() {
        for sample_rate in (6000u32..15500).step_by(100) {
            test_bell_202_encode_decode_at(sample_rate);
        }
    }

    fn test_bell_202_encode_decode_at(sample_rate: u32) {
        let vec: Vec<u8> = hex::decode("82a0aa646a9ce0ae8270989a8c60ae92888a62406303f03e3230323333377a687474703a2f2f7761386c6d662e636f6d0df782").unwrap();

        let iter = bell_202_encode::<f32, _>(vec.into_iter(), sample_rate, 0.75);

        let mut decoder = bell_202_decoder(sample_rate);

        for x in iter {
            if let Some(_x) = decoder.filter(x) {
                //println!("decoded: {:?}", hex::encode(_x));
                return;
            }
        }

        panic!("Unable to decode at {}", sample_rate);
    }

    #[test]
    fn test_bell_202_encode_decode_resample() {
        let vec: Vec<u8> = hex::decode("82a0aa646a9ce0ae8270989a8c60ae92888a62406303f03e3230323333377a687474703a2f2f7761386c6d662e636f6d0df782").unwrap();

        let iter = bell_202_encode::<f32, _>(vec.clone().into_iter(), 44100, 0.75);

        let mut decoder = bell_202_decoder(7200);
        let mut resampler = Downsampler::new(44100, 7200);

        for x in iter {
            if let Some(x) = resampler.filter(x) {
                if let Some(x) = decoder.filter(x) {
                    println!("decoded: {:?}", hex::encode(&x));
                    assert_eq!(vec, x);
                    return;
                }
            }
        }
        panic!("Unable to decode");
    }

    // #[test]
    // fn test_bell_202_decoder() {
    //     use std::fs::File;
    //     use std::path::Path;
    //     use std::str::from_utf8;
    //     const X25: crc::Crc<u16> = crc::Crc::<u16>::new(&crc::CRC_16_IBM_SDLC);
    //
    //     let mut inp_file = File::open(Path::new("/Users/darco/Documents/testcd01.wav")).unwrap();
    //     let (header, data) = wav::read(&mut inp_file).unwrap();
    //
    //     let mut decoder = bell_202_decoder(BELL202_OPTIMAL_SAMPLE_RATE);
    //     let mut downsampler =
    //         Downsampler::<f32>::new(header.sampling_rate, BELL202_OPTIMAL_SAMPLE_RATE);
    //
    //     eprintln!("HEADER: {:?}", header);
    //     let mut framecount = 0u32;
    //     let mut badframecount = 0u32;
    //     let mut drop = false;
    //
    //     match data {
    //         wav::BitDepth::Sixteen(vec) => {
    //             for sample in vec {
    //                 // Remove the stereo
    //                 if header.channel_count == 2 && drop {
    //                     drop = false;
    //                     continue;
    //                 } else {
    //                     drop = true;
    //                 }
    //
    //                 // Convert to floating point
    //                 let sample = sample as f32 / (std::i16::MAX as f32 / 4.0 * 3.0);
    //
    //                 // Downsample
    //                 let sample = if let Some(sample) = downsampler.filter(sample) {
    //                     sample
    //                 } else {
    //                     continue;
    //                 };
    //
    //                 // Decode
    //                 let out = decoder.filter(sample);
    //
    //                 if let Some(frame) = out {
    //                     if frame.len() < 2 {
    //                         continue;
    //                     }
    //
    //                     if X25.checksum(&frame) != 0x0f47 {
    //                         badframecount += 1;
    //                         continue;
    //                     } else {
    //                         framecount += 1;
    //                         //continue;
    //                     }
    //
    //                     let mut addrfield = vec![];
    //
    //                     for x in frame.iter() {
    //                         let x = *x;
    //                         addrfield.push(x >> 1);
    //                         if x & 1 == 1 {
    //                             break;
    //                         }
    //                     }
    //                     let addr_len = addrfield.len();
    //                     let addr_escaped = addrfield
    //                         .into_iter()
    //                         .map(|x| if x.is_ascii() && x > 31 { x } else { b'.' })
    //                         .flat_map(std::ascii::escape_default)
    //                         .collect::<Vec<_>>();
    //                     let addr_str = from_utf8(&addr_escaped).unwrap();
    //
    //                     //println!("GOT FRAME: {}", hex::encode(&frame));
    //
    //                     let escaped: Vec<u8> = frame[addr_len..]
    //                         .iter()
    //                         .map(|&x| if x.is_ascii() && x > 31 { x } else { b'.' })
    //                         .flat_map(std::ascii::escape_default)
    //                         .collect();
    //                     println!("[{}]{}", addr_str, from_utf8(&escaped).unwrap());
    //                 }
    //             }
    //         }
    //         _ => panic!("bad data"),
    //     }
    //     println!("decoded {} frames", framecount);
    //     println!("bad frames: {}", badframecount);
    //     eprintln!("FINISHED PROCESSING");
    //     //panic!();
    // }
}
