/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub trait WireMessage: Sized {
    fn wire_bytes(&self) -> Vec<u8>;

    fn from_wire(bytes: &[u8]) -> Result<(dokime::harmos::operations::StreamItem<Self>, usize), tonic::Status>;
}

impl<T: dokime::harmos::domain::Portable> WireMessage for T {
    fn wire_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        <dokime::harmos::grpc::Transcoded as dokime::harmos::grpc::Wire<T>>::encode(self, &mut bytes).expect("a story message transcodes");
        bytes
    }

    fn from_wire(bytes: &[u8]) -> Result<(dokime::harmos::operations::StreamItem<Self>, usize), tonic::Status> {
        let message_bytes = 16 * 1024 * 1024;
        if bytes.len() > message_bytes {
            return Err(tonic::Status::resource_exhausted("fixture message exceeds wire ceiling"));
        }
        let retained = 64 * 1024 * 1024;
        let budget = std::sync::Arc::new(tokio::sync::Semaphore::new(retained));
        let mut admitted_memory = 0;
        let (value, claim) = <dokime::harmos::grpc::Transcoded as dokime::harmos::grpc::Wire<T>>::decode_admitted(
            harmos_sdk::grpc::DecodeInput::new(bytes),
            |memory| {
                let memory = memory.max(std::mem::size_of::<T>()).max(1);
                let permits =
                    u32::try_from(memory).map_err(|_| tonic::Status::resource_exhausted("fixture decode exceeds permit capacity"))?;
                if memory > retained {
                    return Err(tonic::Status::resource_exhausted("fixture decoded message exceeds byte ceiling"));
                }
                let claim = budget
                    .try_acquire_many_owned(permits)
                    .map_err(|_| tonic::Status::resource_exhausted("fixture decoded message retains byte capacity"))?;
                admitted_memory = memory;
                Ok(claim)
            },
        )?;
        let item =
            dokime::harmos::operations::StreamItem::charged(value, claim).map_err(|error| tonic::Status::internal(error.to_string()))?;
        Ok((item, admitted_memory))
    }
}
