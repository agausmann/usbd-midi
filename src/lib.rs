#![no_std]

use core::convert::TryFrom;

use usb_device::class_prelude::{
    DescriptorWriter, EndpointIn, EndpointOut, InterfaceNumber, UsbBus, UsbBusAllocator, UsbClass,
};

pub struct MidiClass<'a, B>
where
    B: UsbBus,
{
    audio_control_if: InterfaceNumber,
    midi_streaming_if: InterfaceNumber,
    out_ep: EndpointOut<'a, B>,
    in_ep: EndpointIn<'a, B>,
}

impl<'a, B> MidiClass<'a, B>
where
    B: UsbBus,
{
    pub fn new(alloc: &'a UsbBusAllocator<B>) -> Self {
        Self {
            audio_control_if: alloc.interface(),
            midi_streaming_if: alloc.interface(),
            out_ep: alloc.bulk(64),
            in_ep: alloc.bulk(64),
        }
    }
}

impl<'a, B> UsbClass<B> for MidiClass<'a, B>
where
    B: UsbBus,
{
    fn get_configuration_descriptors(
        &self,
        writer: &mut DescriptorWriter,
    ) -> Result<(), usb_device::UsbError> {
        writer.interface(
            self.audio_control_if,
            0x01, // Audio class
            0x01, // Audio control subclass
            0x00, // Protocol (Unused)
        )?;
        // Class-specific AC interface descriptor
        let cs_ac_size_handle = writer.defer_ahead(5..7);
        let cs_ac_start = writer.position();
        writer.write(
            0x24, // CS_INTERFACE
            &[
                0x01, // HEADER subtype
                0x00,
                0x01, // Audio device class specification revision 1.0
                0x00,
                0x00, // Total size of class specific descriptors
                0x01, // Number of streaming interfaces belonging to this control iface:
                self.midi_streaming_if.into(),
            ],
        )?;
        let cs_ac_end = writer.position();
        let cs_ac_size = u16::try_from(cs_ac_end - cs_ac_start).unwrap();
        writer
            .get_deferred_mut(&cs_ac_size_handle)?
            .copy_from_slice(&cs_ac_size.to_le_bytes());

        writer.interface(
            self.midi_streaming_if,
            0x01, // Audio class
            0x03, // Midi streaming subclass
            0x00, // Protocol (Unused)
        )?;
        // Class-specific MS interface descriptor
        let cs_ms_size_handle = writer.defer_ahead(5..7);
        let cs_ms_start = writer.position();
        writer.write(
            0x24, // CS_INTERFACE
            &[
                0x01, // HEADER subtype
                0x00, 0x01, // Midi device class specification revision 1.0
                0x00, 0x00, // Total size of class specific descriptors
            ],
        )?;
        // MIDI IN Jack Descriptor
        writer.write(
            0x24, // CS_INTERFACE
            &[
                0x02, // MIDI_IN_JACK subtype
                0x01, // EMBEDDED
                0x01, // Jack ID
                0x00, // Unused
            ],
        )?;
        // MIDI OUT Jack descriptor
        writer.write(
            0x24, // CS_INTERFACE
            &[
                0x03, // MIDI_OUT_JACK subtype
                0x01, // EMBEDDED
                0x02, // Jack ID
                0x01, // Number of input pins:
                0x01, // Entity ID [1]
                0x01, // Entity output pin number [1]
                0x00, // Unused
            ],
        )?;

        writer.endpoint(&self.out_ep)?;
        // Class-specific
        writer.write(
            0x25, // CS_ENDPOINT
            &[
                0x01, // MS_GENERAL subtype
                0x01, // Number of embedded MIDI IN jacks:
                0x01, // Jack ID [1]
            ],
        )?;

        writer.endpoint(&self.in_ep)?;
        // Class-specific
        writer.write(
            0x25, // CS_ENDPOINT
            &[
                0x01, // MS_GENERAL subtype
                0x01, // Number of embedded MIDI OUT jacks:
                0x02, // Jack ID [1]
            ],
        )?;
        let cs_ms_end = writer.position();
        let cs_ms_size = u16::try_from(cs_ms_end - cs_ms_start).unwrap();
        writer
            .get_deferred_mut(&cs_ms_size_handle)?
            .copy_from_slice(&cs_ms_size.to_le_bytes());

        Ok(())
    }
}
