pub fn build_connect(client_id: &[u8]) -> Vec<u8> {
    let remaining_length = 10 + client_id.len();

    let mut connect_message = vec![
        // Fixed Header
        0x10, // 1- CONNECT header
        10,  // 2- Remaining length

        // Variable Header
        0x0, // 1- Length MSB (0)
        0x04, // 2- Length LSB (4)
        'M' as u8, // 3
        'Q' as u8, // 4
        'T' as u8, // 5
        'T' as u8, // 6
        0x04, // 7- Protocol level (4 == 3.1.1)
        0x02, // 8- Flags: Clean session
        0x0, // 9- Keep Alive MSB
        10, // 10- Keep Alive LSB
    ];

    connect_message.extend_from_slice(client_id);
    connect_message[1] = remaining_length as u8;

    return connect_message;
}