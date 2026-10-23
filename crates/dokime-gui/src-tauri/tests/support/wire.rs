/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */
pub trait WireMessage: Sized {
    fn wire_bytes(&self) -> Vec<u8>;

    fn from_wire(bytes: &[u8]) -> Result<Self, dokime::harmos::MessageError>;
}

impl<T: dokime::harmos::domain::Portable> WireMessage for T {
    fn wire_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        <dokime::harmos::grpc::Transcoded as dokime::harmos::grpc::Wire<T>>::encode(self, &mut bytes).expect("a story message transcodes");
        bytes
    }

    fn from_wire(bytes: &[u8]) -> Result<Self, dokime::harmos::MessageError> {
        <dokime::harmos::grpc::Transcoded as dokime::harmos::grpc::Wire<T>>::decode(harmos_sdk::grpc::DecodeInput::new(bytes))
    }
}

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
