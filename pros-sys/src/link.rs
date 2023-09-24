#[doc(hidden)]
pub const E_LINK_RECIEVER: core::ffi::c_uint = 0;
// PROS PLS FIX 🥺
pub use E_LINK_RECIEVER as E_LINK_RECEIVER;
pub const E_LINK_TRANSMITTER: core::ffi::c_uint = 1;
pub const E_LINK_RX: core::ffi::c_uint = E_LINK_RECEIVER;
pub const E_LINK_TX: core::ffi::c_uint = E_LINK_TRANSMITTER;
pub type link_type_e_t = core::ffi::c_uint;

extern "C" {
    /**
    Initializes a link on a radio port, with an indicated type. There might be a
    1 to 2 second delay from when this function is called to when the link is initializes.
    PROS currently only supports the use of one radio per brain.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a radio.
    ENXIO - The sensor is still calibrating, or no link is connected via the radio.

    \param port
         The port of the radio for the intended link.
    \param link_id
         Unique link ID in the form of a string, needs to be different from other links in
         the area.
    \param type
         Indicates whether the radio link on the brain is a transmitter or receiver,
         with the transmitter having double the transmitting bandwidth as the receiving
         end (1040 bytes/s vs 520 bytes/s).

    \return PROS_ERR if initialization fails, 1 if the initialization succeeds.
    */
    pub fn link_init(port: u8, link_id: *const core::ffi::c_char, r#type: link_type_e_t) -> u32;
    /**
    Initializes a link on a radio port, with an indicated type and the ability for
    vexlink to override the controller radio. There might be a 1 to 2 second delay
    from when this function is called to when the link is initializes.
    PROS currently only supports the use of one radio per brain.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a radio.
    ENXIO - The sensor is still calibrating, or no link is connected via the radio.

    \param port
         The port of the radio for the intended link.
    \param link_id
         Unique link ID in the form of a string, needs to be different from other links in
         the area.
    \param type
         Indicates whether the radio link on the brain is a transmitter or receiver,
         with the transmitter having double the transmitting bandwidth as the receiving
         end (1040 bytes/s vs 520 bytes/s).

    \return PROS_ERR if initialization fails, 1 if the initialization succeeds.
    */
    pub fn link_init_override(
        port: u8,
        link_id: *const core::ffi::c_char,
        r#type: link_type_e_t,
    ) -> u32;
    /**
    Checks if a radio link on a port is active or not.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a radio.
    ENXIO - The sensor is still calibrating, or no link is connected via the radio.

    \param port
         The port of the radio for the intended link.

    \return If a radio is connected to a port and it's connected to a link.
    */
    pub fn link_connected(port: u8) -> bool;
    /**
    Returns the bytes of data available to be read

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a radio.
    ENXIO - The sensor is still calibrating, or no link is connected via the radio.

    \param port
         The port of the radio for the intended link.

    \return PROS_ERR if port is not a link/radio, else the bytes available to be
    read by the user.
    */
    pub fn link_raw_receivable_size(port: u8) -> u32;
    /**
    Returns the bytes of data available in transmission buffer.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a radio.
    ENXIO - The sensor is still calibrating, or no link is connected via the radio.

    \param port
         The port of the radio for the intended link.

    \return PROS_ERR if port is not a link/radio,
    */
    pub fn link_raw_transmittable_size(port: u8) -> u32;
    /**
    Send raw serial data through vexlink.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a radio.
    ENXIO - The sensor is still calibrating, or no link is connected via the radio.
    EBUSY - The transmitter buffer is still busy with a previous transmission, and there is no
    room in the FIFO buffer (queue) to transmit the data.
    EINVAL - The data given is NULL

    \param port
         The port of the radio for the intended link.
    \param data
         Buffer with data to send
    \param data_size
         Bytes of data to be read to the destination buffer

    \return PROS_ERR if port is not a link, and the successfully transmitted
    data size if it succeeded.
    */
    pub fn link_transmit_raw(port: u8, data: *const core::ffi::c_void, data_size: u16) -> u32;
    /**
    Receive raw serial data through vexlink.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a radio.
    ENXIO - The sensor is still calibrating, or no link is connected via the radio.
    EINVAL - The destination given is NULL, or the size given is larger than the FIFO buffer
    or destination buffer.

    \param port
         The port of the radio for the intended link.
    \param dest
         Destination buffer to read data to
    \param data_size
         Bytes of data to be read to the destination buffer

    \return PROS_ERR if port is not a link, and the successfully received
    data size if it succeeded.
    */
    pub fn link_receive_raw(port: u8, dest: *mut core::ffi::c_void, data_size: u16) -> u32;
    /**
    Send packeted message through vexlink, with a checksum and start byte.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a radio.
    ENXIO - The sensor is still calibrating, or no link is connected via the radio.
    EBUSY - The transmitter buffer is still busy with a previous transmission, and there is no
    room in the FIFO buffer (queue) to transmit the data.
    EINVAL - The data given is NULL

    \param port
         The port of the radio for the intended link.
    \param data
         Buffer with data to send
    \param data_size
         Bytes of data to be read to the destination buffer

    \return PROS_ERR if port is not a link, and the successfully transmitted
    data size if it succeeded.
    */
    pub fn link_transmit(port: u8, data: *const core::ffi::c_void, data_size: u16) -> u32;
    /**
    Receive packeted message through vexlink, with a checksum and start byte.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a radio.
    ENXIO - The sensor is still calibrating, or no link is connected via the radio.
    EINVAL - The destination given is NULL, or the size given is larger than the FIFO buffer
    or destination buffer.
    EBADMSG - Protocol error related to start byte, data size, or checksum.

    \param port
         The port of the radio for the intended link.
    \param dest
         Destination buffer to read data to
    \param data_size
         Bytes of data to be read to the destination buffer

    \return PROS_ERR if port is not a link or protocol error, and the successfully
    transmitted data size if it succeeded.
    */
    pub fn link_receive(port: u8, dest: *mut core::ffi::c_void, data_size: u16) -> u32;
    /**
    Clear the receive buffer of the link, and discarding the data.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a radio.
    ENXIO - The sensor is still calibrating, or no link is connected via the radio.

    \param port
         The port of the radio for the intended link.

    \return PROS_ERR if port is not a link, and the successfully received
    data size if it succeeded.
    */
    pub fn link_clear_receive_buf(port: u8) -> u32;
}
