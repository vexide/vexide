use core::ffi::c_uint;

pub const VISION_OBJECT_ERR_SIG: u16 = 255;
pub const VISION_FOV_WIDTH: c_uint = 316;
pub const VISION_FOV_HEIGHT: c_uint = 212;

pub const E_VISION_OBJECT_NORMAL: c_uint = 0;
pub const E_VISION_OBJECT_COLOR_CODE: c_uint = 1;
pub const E_VISION_OBJECT_LINE: c_uint = 2;
/**
 * This enumeration defines the different types of objects
 * that can be detected by the Vision Sensor
 */
pub type vision_object_type_e_t = c_uint;

/**
 * This structure contains the parameters used by the Vision Sensor
 * to detect objects.
 */
#[repr(packed, C)]
pub struct vision_signature_s_t {
    pub id: u8,
    pub _pad: [u8; 3],
    pub range: f32,
    pub u_min: i32,
    pub u_max: i32,
    pub u_mean: i32,
    pub v_min: i32,
    pub v_max: i32,
    pub v_mean: i32,
    pub rgb: u32,
    pub r#type: u32,
}

/**
 * Color codes are just signatures with multiple IDs and a different type.
 */
pub type vision_color_code_t = u16;

/**
 * This structure contains a descriptor of an object detected
 * by the Vision Sensor
 */
#[repr(packed, C)]
pub struct vision_object_s_t {
    pub signature: u16,
    pub r#type: vision_object_type_e_t,
    pub left_coord: i16,
    pub top_coord: i16,
    pub width: i16,
    pub height: i16,
    pub angle: i16,
    pub x_middle_coord: i16,
    pub y_middle_coord: i16,
}

pub const E_VISION_ZERO_TOPLEFT: c_uint = 0;
pub const E_VISION_ZERO_CENTER: c_uint = 1;
pub type vision_zero_e_t = c_uint;

extern "C" {
    /**
    Clears the vision sensor LED color, resetting it back to its default behavior,
    displaying the most prominent object signature color.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a vision sensor

    \param port
           The V5 port number from 1-21

    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn vision_clear_led(port: u8) -> i32;
    /**
    Creates a signature from the vision sensor utility

    \param id
           The signature ID
    \param u_min
           Minimum value on U axis
    \param u_max
           Maximum value on U axis
    \param u_mean
           Mean value on U axis
    \param v_min
           Minimum value on V axis
    \param v_max
           Maximum value on V axis
    \param v_mean
           Mean value on V axis
    \param range
           Scale factor
    \param type
           Signature type

    \return A vision_signature_s_t that can be set using vision_set_signature
    */
    pub fn vision_signature_from_utility(
        id: i32,
        u_min: i32,
        u_max: i32,
        u_mean: i32,
        v_min: i32,
        v_max: i32,
        v_mean: i32,
        range: f32,
        r#type: i32,
    ) -> vision_signature_s_t;
    /**
    Creates a color code that represents a combination of the given signature
    IDs. If fewer than 5 signatures are to be a part of the color code, pass 0
    for the additional function parameters.

    This function uses the following values of errno when an error state is
    reached:
    EINVAL - Fewer than two signatures have been provided or one of the
             signatures is out of its [1-7] range (or 0 when omitted).

    \param port
           The V5 port number from 1-21
    \param sig_id1
           The first signature id [1-7] to add to the color code
    \param sig_id2
           The second signature id [1-7] to add to the color code
    \param sig_id3
           The third signature id [1-7] to add to the color code
    \param sig_id4
           The fourth signature id [1-7] to add to the color code
    \param sig_id5
           The fifth signature id [1-7] to add to the color code

    \return A vision_color_code_t object containing the color code information.
    */
    pub fn vision_create_color_code(
        port: u8,
        sig_id1: u32,
        sig_id2: u32,
        sig_id3: u32,
        sig_id4: u32,
        sig_id5: u32,
    ) -> vision_color_code_t;
    /**
    Gets the nth largest object according to size_id.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a vision sensor
    EDOM - size_id is greater than the number of available objects.
    EHOSTDOWN - Reading the vision sensor failed for an unknown reason.

    \param port
           The V5 port number from 1-21
    \param size_id
           The object to read from a list roughly ordered by object size
           (0 is the largest item, 1 is the second largest, etc.)

    \return The vision_object_s_t object corresponding to the given size id, or
    PROS_ERR if an error occurred.
    */
    pub fn vision_get_by_size(port: u8, size_id: u32) -> vision_object_s_t;
    /**
    Gets the nth largest object of the given signature according to size_id.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a vision sensor
    EINVAL - sig_id is outside the range [1-8]
    EDOM - size_id is greater than the number of available objects.
    EAGAIN - Reading the vision sensor failed for an unknown reason.

    \param port
           The V5 port number from 1-21
    \param size_id
           The object to read from a list roughly ordered by object size
           (0 is the largest item, 1 is the second largest, etc.)
    \param signature
           The signature ID [1-7] for which an object will be returned.

    \return The vision_object_s_t object corresponding to the given signature and
    size_id, or PROS_ERR if an error occurred.
    */
    pub fn vision_get_by_sig(port: u8, size_id: u32, sig_id: u32) -> vision_object_s_t;
    /**
    Gets the nth largest object of the given color code according to size_id.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a vision sensor
    EAGAIN - Reading the vision sensor failed for an unknown reason.

    \param port
           The V5 port number from 1-21
    \param size_id
           The object to read from a list roughly ordered by object size
           (0 is the largest item, 1 is the second largest, etc.)
    \param color_code
           The vision_color_code_t for which an object will be returned

    \return The vision_object_s_t object corresponding to the given color code
    and size_id, or PROS_ERR if an error occurred.
    */
    pub fn vision_get_by_code(
        port: u8,
        size_id: u32,
        code: vision_color_code_t,
    ) -> vision_object_s_t;
    /**
    Gets the exposure parameter of the Vision Sensor. See
    <https://pros.cs.purdue.edu/v5/tutorials/topical/vision.html#exposure-setting>
    for more details.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a vision sensor

    \param port
           The V5 port number from 1-21

    \return The current exposure setting from \[0,150], PROS_ERR if an error
    occurred
    */
    pub fn vision_get_exposure(port: u8) -> i32;
    /**
    Gets the number of objects currently detected by the Vision Sensor.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a vision sensor

    \param port
           The V5 port number from 1-21

    \return The number of objects detected on the specified vision sensor.
    Returns PROS_ERR if the port was invalid or an error occurred.
    */
    pub fn vision_get_object_count(port: u8) -> i32;
    /**
    Get the white balance parameter of the Vision Sensor.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a vision sensor

    \param port
                The V5 port number from 1-21

    \return The current RGB white balance setting of the sensor
    */
    pub fn vision_get_white_balance(port: u8) -> i32;
    /**
    Prints the contents of the signature as an initializer list to the terminal.

    \param sig
           The signature for which the contents will be printed

    \return 1 if no errors occurred, PROS_ERR otherwise
    */
    pub fn vision_print_signature(sig: vision_signature_s_t) -> i32;
    /**
    Reads up to object_count object descriptors into object_arr.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21), or
             fewer than object_count number of objects were found.
    ENODEV - The port cannot be configured as a vision sensor
    EDOM - size_id is greater than the number of available objects.

    \param port
           The V5 port number from 1-21
    \param size_id
           The object to read from a list roughly ordered by object size
           (0 is the largest item, 1 is the second largest, etc.)
    \param object_count
           The number of objects to read
    \param\[out] object_arr
                A pointer to copy the objects into

    \return The number of object signatures copied. This number will be less than
    object_count if there are fewer objects detected by the vision sensor.
    Returns PROS_ERR if the port was invalid, an error occurred, or fewer objects
    than size_id were found. All objects in object_arr that were not found are
    given VISION_OBJECT_ERR_SIG as their signature.
    */
    pub fn vision_read_by_size(
        port: u8,
        size_id: u32,
        object_count: u32,
        object_arr: *mut vision_object_s_t,
    ) -> i32;
    /**
    Reads up to object_count object descriptors into object_arr.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21), or
             fewer than object_count number of objects were found.
    ENODEV - The port cannot be configured as a vision sensor
    EDOM - size_id is greater than the number of available objects.

    \param port
           The V5 port number from 1-21
    \param object_count
           The number of objects to read
    \param size_id
           The object to read from a list roughly ordered by object size
           (0 is the largest item, 1 is the second largest, etc.)
    \param signature
           The signature ID [1-7] for which objects will be returned.
    \param\[out] object_arr
                A pointer to copy the objects into

    \return The number of object signatures copied. This number will be less than
    object_count if there are fewer objects detected by the vision sensor.
    Returns PROS_ERR if the port was invalid, an error occurred, or fewer objects
    than size_id were found. All objects in object_arr that were not found are
    given VISION_OBJECT_ERR_SIG as their signature.
    */
    pub fn vision_read_by_sig(
        port: u8,
        size_id: u32,
        sig_id: u32,
        object_count: u32,
        object_arr: *mut vision_object_s_t,
    ) -> i32;
    /**
    Reads up to object_count object descriptors into object_arr.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21), or
             fewer than object_count number of objects were found.
    ENODEV - The port cannot be configured as a vision sensor

    \param port
           The V5 port number from 1-21
    \param object_count
           The number of objects to read
    \param size_id
           The object to read from a list roughly ordered by object size
           (0 is the largest item, 1 is the second largest, etc.)
    \param color_code
           The vision_color_code_t for which objects will be returned
    \param\[out] object_arr
                A pointer to copy the objects into

    \return The number of object signatures copied. This number will be less than
    object_count if there are fewer objects detected by the vision sensor.
    Returns PROS_ERR if the port was invalid, an error occurred, or fewer objects
    than size_id were found. All objects in object_arr that were not found are
    given VISION_OBJECT_ERR_SIG as their signature.
    */
    pub fn vision_read_by_code(
        port: u8,
        size_id: u32,
        code: vision_color_code_t,
        object_count: u32,
        object_arr: *mut vision_object_s_t,
    ) -> i32;
    /**
    Gets the object detection signature with the given id number.

    \param port
           The V5 port number from 1-21
    \param signature_id
           The signature id to read

    \return A vision_signature_s_t containing information about the signature.
    */
    pub fn vision_get_signature(port: u8, signature_id: u8) -> vision_signature_s_t;
    /**
    Stores the supplied object detection signature onto the vision sensor.

    NOTE: This saves the signature in volatile memory, and the signature will be
    lost as soon as the sensor is powered down.

    \param port
           The V5 port number from 1-21
    \param signature_id
           The signature id to store into
    \param\[in] signature_ptr
               A pointer to the signature to save

    \return 1 if no errors occurred, PROS_ERR otherwise
    */
    pub fn vision_set_signature(
        port: u8,
        signature_id: u8,
        signature_ptr: *const vision_signature_s_t,
    ) -> i32;
    /**
    Enables/disables auto white-balancing on the Vision Sensor.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a vision sensor
    EINVAL - enable was not 0 or 1

    \param port
                The V5 port number from 1-21
    \param enabled
                Pass 0 to disable, 1 to enable

    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn vision_set_auto_white_balance(port: u8, enable: u8) -> i32;
    /**
    Sets the exposure parameter of the Vision Sensor. See
    <https://pros.cs.purdue.edu/v5/tutorials/topical/vision.html#exposure-setting>
    for more details.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a vision sensor

    \param port
           The V5 port number from 1-21
    \param percent
           The new exposure setting from \[0,150]

    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn vision_set_exposure(port: u8, exposure: u8) -> i32;
    /**
    Sets the vision sensor LED color, overriding the automatic behavior.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a vision sensor

    \param port
           The V5 port number from 1-21
    \param rgb
           An RGB code to set the LED to

    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn vision_set_led(port: u8, rgb: i32) -> i32;
    /**
    Sets the white balance parameter of the Vision Sensor.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a vision sensor

    \param port
                The V5 port number from 1-21
    \param rgb
           The new RGB white balance setting of the sensor

    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn vision_set_white_balance(port: u8, rgb: i32) -> i32;
    /**
    Sets the (0,0) coordinate for the Field of View.

    This will affect the coordinates returned for each request for a
    vision_object_s_t from the sensor, so it is recommended that this function
    only be used to configure the sensor at the beginning of its use.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a vision sensor

    \param port
                The V5 port number from 1-21
    \param zero_point
           One of vision_zero_e_t to set the (0,0) coordinate for the FOV

    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn vision_set_zero_point(port: u8, zero_point: vision_zero_e_t) -> i32;
    /**
    Sets the Wi-Fi mode of the Vision sensor

    This functions uses the following values of errno when an error state is
    reached:
    ENXIO - The given port is not within the range of V5 ports (1-21)
    EACCESS - Another resource is currently trying to access the port

    \param port
           The V5 port number from 1-21
    \param enable
           Disable Wi-Fi on the Vision sensor if 0, enable otherwise (e.g. 1)

    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn vision_set_wifi_mode(port: u8, mode: u8) -> i32;
}
