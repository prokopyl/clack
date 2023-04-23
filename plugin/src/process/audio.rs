mod input;
mod output;
mod pair;
mod sample_type;

pub use input::*;
pub use output::*;
pub use pair::*;
pub use sample_type::SampleType;

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::prelude::Audio;
    use clack_common::process::ConstantMask;
    use clack_host::prelude::*;

    fn get_audio<'a, const N: usize>(
        ins: &'a mut [[f32; N]],
        outs: &'a mut [[f32; N]],
        input_ports: &'a mut AudioPorts,
        output_ports: &'a mut AudioPorts,
    ) -> Audio<'a> {
        let input_buffers = input_ports.with_input_buffers([AudioPortBuffer {
            latency: 0,
            channels: AudioPortBufferType::f32_input_only(
                ins.iter_mut().map(InputChannel::variable),
            ),
        }]);

        let output_buffers = output_ports.with_output_buffers([AudioPortBuffer {
            latency: 0,
            channels: AudioPortBufferType::f32_output_only(
                outs.iter_mut().map(|b| b.as_mut_slice()),
            ),
        }]);

        Audio {
            inputs: input_buffers.as_raw_buffers(),
            frames_count: input_buffers
                .min_channel_buffer_length()
                .min(output_buffers.min_channel_buffer_length()) as u32,
            outputs: output_buffers.into_raw_buffers(),
        }
    }

    #[test]
    fn can_iterate_on_io() {
        let mut ins = [[1f32; 4]; 2];
        let mut outs = [[0f32; 4]; 2];

        let mut input_ports = AudioPorts::with_capacity(2, 1);
        let mut output_ports = AudioPorts::with_capacity(2, 1);

        let mut audio = get_audio(&mut ins, &mut outs, &mut input_ports, &mut output_ports);
        let mut ports = audio.port_pairs();
        let mut port = ports.next().unwrap();
        assert!(ports.next().is_none());

        let channels = port.channel_pairs().unwrap().into_f32().unwrap();
        let mut constant_mask = ConstantMask::FULLY_CONSTANT;
        let mut total = 0;

        for channel in channels {
            let ChannelPair::InputOutput(i, o) = channel else { panic!("Expected I/O channel") };
            o.copy_from_slice(i);
            total += 1;
            constant_mask.set_channel_constant(total, true);
        }

        assert_eq!(total, 2);
        assert_eq!(ins, outs);
    }
}
