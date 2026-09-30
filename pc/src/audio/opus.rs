use std::io::{self, Error, ErrorKind};

pub struct OpusDecoderWrapper {
    decoder: opus_decoder::OpusDecoder,
    output_buf: Vec<f32>,
}

impl OpusDecoderWrapper {
    pub fn new() -> io::Result<Self> {
        let decoder = opus_decoder::OpusDecoder::new(48000, 1)
            .map_err(|e| Error::other(format!("OpusDecoder init error: {:?}", e)))?;

        Ok(Self {
            decoder,
            output_buf: vec![0.0f32; 5760], // up to 120 ms at 48 kHz
        })
    }

    /// Decodes an Opus packet into i16 PCM samples.
    pub fn decode(&mut self, opus_data: &[u8], out_pcm: &mut Vec<i16>) -> io::Result<usize> {
        out_pcm.clear();
        match self
            .decoder
            .decode_float(opus_data, &mut self.output_buf, false)
        {
            Ok(samples_decoded) => {
                out_pcm.reserve(samples_decoded);
                for &s in &self.output_buf[..samples_decoded] {
                    // Clamp float (-1.0 to 1.0) and convert to i16
                    let clamped = s.clamp(-1.0, 1.0);
                    let val = (clamped * 32767.0).round() as i16;
                    out_pcm.push(val);
                }
                Ok(samples_decoded)
            }
            Err(e) => Err(Error::new(
                ErrorKind::InvalidData,
                format!("Opus decode error: {:?}", e),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opus_decoder_creation() {
        let dec = OpusDecoderWrapper::new();
        assert!(dec.is_ok());
    }
}
