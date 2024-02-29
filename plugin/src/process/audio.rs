//! Various types related to accessing [`Audio`](super::Audio) buffers.

mod error;
mod input;
mod output;
mod pair;
mod sample_type;

pub use error::BufferError;
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
                .frames_count()
                .min(output_buffers.frames_count()),
            outputs: output_buffers.into_raw_buffers(),
        }
    }

    #[test]
    fn can_iterate_on_pairs() {
        let mut ins = [[1f32; 4]; 2];
        let mut outs = [[0f32; 4]; 2];

        let mut input_ports = AudioPorts::with_capacity(2, 1);
        let mut output_ports = AudioPorts::with_capacity(2, 1);

        let mut audio = get_audio(&mut ins, &mut outs, &mut input_ports, &mut output_ports);
        let mut ports = audio.port_pairs();
        assert_eq!(ports.len(), 1);
        assert_eq!(ports.size_hint(), (1, Some(1)));
        let mut port = ports.next().unwrap();
        assert!(ports.next().is_none());

        let mut channels = port.channels().unwrap().into_f32().unwrap();
        assert_eq!(channels.iter_mut().len(), 2);
        assert_eq!(channels.iter_mut().size_hint(), (2, Some(2)));
        let mut constant_mask = ConstantMask::FULLY_CONSTANT;
        let mut total = 0;

        for channel in channels {
            let ChannelPair::InputOutput(i, o) = channel else {
                panic!("Expected I/O channel")
            };
            o.copy_from_slice(i);
            total += 1;
            constant_mask.set_channel_constant(total, false);
        }

        port.output().unwrap().set_constant_mask(constant_mask);

        assert_eq!(total, 2);
        assert_eq!(ins, outs);
    }

    #[test]
    fn can_access_pairs_with_indexes() {
        let mut ins = [[1f32; 4]; 2];
        let mut outs = [[0f32; 4]; 2];

        let mut input_ports = AudioPorts::with_capacity(2, 1);
        let mut output_ports = AudioPorts::with_capacity(2, 1);

        let mut audio = get_audio(&mut ins, &mut outs, &mut input_ports, &mut output_ports);
        assert_eq!(audio.port_pair_count(), 1);

        let mut port = audio.port_pair(0).unwrap();
        assert_eq!(port.channel_pair_count(), 2);

        let mut channels = port.channels().unwrap().into_f32().unwrap();
        assert_eq!(channels.channel_pair_count(), 2);

        let mut constant_mask = ConstantMask::FULLY_CONSTANT;

        for i in 0..port.channel_pair_count() {
            let channel = channels.channel_pair(i).unwrap();
            let ChannelPair::InputOutput(input, output) = channel else {
                panic!("Expected I/O channel")
            };
            output.copy_from_slice(input);

            constant_mask.set_channel_constant(i as u64, false);
        }

        port.output().unwrap().set_constant_mask(constant_mask);

        assert_eq!(ins, outs);
    }

    #[test]
    fn can_iterate_on_io() {
        let mut ins = [[1f32; 4]; 2];
        let mut outs = [[0f32; 4]; 2];

        let mut input_ports = AudioPorts::with_capacity(2, 1);
        let mut output_ports = AudioPorts::with_capacity(2, 1);

        let mut audio = get_audio(&mut ins, &mut outs, &mut input_ports, &mut output_ports);

        let mut ports = audio.input_ports();
        assert_eq!(ports.len(), 1);
        assert_eq!(ports.size_hint(), (1, Some(1)));
        let port = ports.next().unwrap();
        assert!(ports.next().is_none());

        let channels = port.channels().unwrap().into_f32().unwrap();
        assert_eq!(channels.iter().len(), 2);
        assert_eq!(channels.iter().size_hint(), (2, Some(2)));
        let mut total = 0;

        for channel in channels {
            assert!(channel.iter().all(|f| *f == 1.0));
            total += 1;
        }

        assert_eq!(total, 2);

        let mut ports = audio.output_ports();
        assert_eq!(ports.len(), 1);
        assert_eq!(ports.size_hint(), (1, Some(1)));
        let mut port = ports.next().unwrap();
        assert!(ports.next().is_none());

        let mut channels = port.channels().unwrap().into_f32().unwrap();
        assert_eq!(channels.iter_mut().len(), 2);
        assert_eq!(channels.iter_mut().size_hint(), (2, Some(2)));
        let mut total = 0;

        for channel in channels {
            channel.fill(1.0);
            total += 1;
        }

        assert_eq!(total, 2);

        assert_eq!(ins, outs);
    }

    #[test]
    fn can_access_io_with_indexes() {
        let mut ins = [[1f32; 4]; 2];
        let mut outs = [[0f32; 4]; 2];

        let mut input_ports = AudioPorts::with_capacity(2, 1);
        let mut output_ports = AudioPorts::with_capacity(2, 1);

        let mut audio = get_audio(&mut ins, &mut outs, &mut input_ports, &mut output_ports);
        assert_eq!(audio.port_pair_count(), 1);

        let port = audio.input_port(0).unwrap();
        assert_eq!(port.channel_count(), 2);

        let channels = port.channels().unwrap().into_f32().unwrap();
        assert_eq!(channels.channel_count(), 2);

        for i in 0..port.channel_count() {
            let channel = channels.channel(i).unwrap();
            assert!(channel.iter().all(|f| *f == 1.0));
        }

        let mut port = audio.output_port(0).unwrap();
        assert_eq!(port.channel_count(), 2);

        let mut channels = port.channels().unwrap().into_f32().unwrap();
        assert_eq!(channels.channel_count(), 2);

        let constant_mask = ConstantMask::FULLY_CONSTANT;

        for i in 0..port.channel_count() {
            let channel = channels.channel_mut(i).unwrap();
            channel.fill(1.0);
        }

        port.set_constant_mask(constant_mask);

        assert_eq!(ins, outs);
    }
}
