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
macro_rules! product_error {
    ($error:ty, $formatter:ident { $($pattern:pat => $display:expr),+ $(,)? }) => {
        impl std::fmt::Display for $error {
            fn fmt(&self, $formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $($pattern => $display),+
                }
            }
        }

        impl std::error::Error for $error {}
    };
}

macro_rules! json_message {
    ($message:ty) => {
        impl harmos::Message for $message {
            fn encode_to_vec(&self) -> Vec<u8> {
                let json = serde_json::to_string(self).expect("an authored payload serializes");
                harmos::Message::encode_to_vec(&json)
            }

            fn encoded_len(&self) -> usize {
                let json = serde_json::to_string(self).expect("an authored payload serializes");
                harmos::Message::encoded_len(&json)
            }

            fn decode(bytes: &[u8]) -> Result<Self, harmos::MessageError> {
                let json = <String as harmos::Message>::decode(bytes)?;
                serde_json::from_str(&json)
                    .map_err(|_| harmos::MessageError::borrowed("invalid authored JSON payload"))
            }
        }

        impl harmos::schema::Described for $message {
            const FIELDS: &'static [harmos::schema::Field] = &[harmos::schema::Field {
                name: std::borrow::Cow::Borrowed("json"),
                tag: 1,
                kind: harmos::schema::Kind::String,
                cardinality: harmos::schema::Cardinality::Required,
            }];
        }

        impl prost::Message for $message {
            fn encode_raw(&self, buffer: &mut impl prost::bytes::BufMut) {
                let json = serde_json::to_vec(self).expect("an authored payload serializes");

                prost::encoding::bytes::encode(1, &json, buffer);
            }

            #[allow(deprecated)]
            fn merge_field(
                &mut self,
                tag: u32,
                wire_type: prost::encoding::WireType,
                buffer: &mut impl prost::bytes::Buf,
                context: prost::encoding::DecodeContext,
            ) -> ::core::result::Result<(), prost::DecodeError> {
                match tag {
                    1 => {
                        let mut json = Vec::new();
                        prost::encoding::bytes::merge(wire_type, &mut json, buffer, context)?;
                        *self = serde_json::from_slice(&json)
                            .map_err(|error| prost::DecodeError::new(error.to_string()))?;

                        Ok(())
                    }
                    _ => prost::encoding::skip_field(wire_type, tag, buffer, context),
                }
            }

            fn encoded_len(&self) -> usize {
                let json = serde_json::to_vec(self).expect("an authored payload serializes");

                prost::encoding::bytes::encoded_len(1, &json)
            }

            fn clear(&mut self) {
                *self = Self::default();
            }
        }
    };
}

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
