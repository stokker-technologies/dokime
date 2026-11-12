/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Services;
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl v1::notes_server::Notes for Services {
    async fn create_note(
        &self, request: tonic::Request<v1::NotesCreateNoteRequest>,
    ) -> Result<tonic::Response<v1::NoteReceipt>, tonic::Status> {
        let reply = dokime_operations_next::client::notes_create_note::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunCreateNoteStream = dokime_operations_next::client::Output<v1::NotesCreateNoteRunOutput>;
    async fn run_create_note(
        &self, request: tonic::Request<tonic::Streaming<v1::NotesCreateNoteRunInput>>,
    ) -> Result<tonic::Response<Self::RunCreateNoteStream>, tonic::Status> {
        let output = dokime_operations_next::client::run_call::unary(
            &self.context,
            request.into_inner(),
            |input| {
                input.value.map(|value| match value {
                    v1::notes_create_note_run_input::Value::Request(request) => {
                        dokime_operations_next::client::run_call::Input::Request(request)
                    }
                    v1::notes_create_note_run_input::Value::Cancel(_) => dokime_operations_next::client::run_call::Input::Cancel,
                    v1::notes_create_note_run_input::Value::Finish(_) => dokime_operations_next::client::run_call::Input::Finish,
                })
            },
            |context, request| async move { dokime_operations_next::client::notes_create_note::handle(&context, request).await },
            |reply| v1::NotesCreateNoteRunOutput {
                value: Some(v1::notes_create_note_run_output::Value::Response(reply)),
            },
        )
        .await
        .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(output))
    }
    async fn edit_note(
        &self, request: tonic::Request<v1::NotesEditNoteRequest>,
    ) -> Result<tonic::Response<v1::EditNoteResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::notes_edit_note::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunEditNoteStream = dokime_operations_next::client::Output<v1::NotesEditNoteRunOutput>;
    async fn run_edit_note(
        &self, request: tonic::Request<tonic::Streaming<v1::NotesEditNoteRunInput>>,
    ) -> Result<tonic::Response<Self::RunEditNoteStream>, tonic::Status> {
        let output = dokime_operations_next::client::run_call::unary(
            &self.context,
            request.into_inner(),
            |input| {
                input.value.map(|value| match value {
                    v1::notes_edit_note_run_input::Value::Request(request) => {
                        dokime_operations_next::client::run_call::Input::Request(request)
                    }
                    v1::notes_edit_note_run_input::Value::Cancel(_) => dokime_operations_next::client::run_call::Input::Cancel,
                    v1::notes_edit_note_run_input::Value::Finish(_) => dokime_operations_next::client::run_call::Input::Finish,
                })
            },
            |context, request| async move { dokime_operations_next::client::notes_edit_note::handle(&context, request).await },
            |reply| v1::NotesEditNoteRunOutput {
                value: Some(v1::notes_edit_note_run_output::Value::Response(reply)),
            },
        )
        .await
        .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(output))
    }
    async fn list_notes(
        &self, request: tonic::Request<v1::NotesListNotesRequest>,
    ) -> Result<tonic::Response<v1::ListNotesResponse>, tonic::Status> {
        let reply = dokime_operations_next::client::notes_list_notes::handle(&self.context, request.into_inner())
            .await
            .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(reply))
    }
    type RunListNotesStream = dokime_operations_next::client::Output<v1::NotesListNotesRunOutput>;
    async fn run_list_notes(
        &self, request: tonic::Request<tonic::Streaming<v1::NotesListNotesRunInput>>,
    ) -> Result<tonic::Response<Self::RunListNotesStream>, tonic::Status> {
        let output = dokime_operations_next::client::run_call::unary(
            &self.context,
            request.into_inner(),
            |input| {
                input.value.map(|value| match value {
                    v1::notes_list_notes_run_input::Value::Request(request) => {
                        dokime_operations_next::client::run_call::Input::Request(request)
                    }
                    v1::notes_list_notes_run_input::Value::Cancel(_) => dokime_operations_next::client::run_call::Input::Cancel,
                    v1::notes_list_notes_run_input::Value::Finish(_) => dokime_operations_next::client::run_call::Input::Finish,
                })
            },
            |context, request| async move { dokime_operations_next::client::notes_list_notes::handle(&context, request).await },
            |reply| v1::NotesListNotesRunOutput {
                value: Some(v1::notes_list_notes_run_output::Value::Response(reply)),
            },
        )
        .await
        .map_err(dokime_operations_next::client::status)?;
        Ok(tonic::Response::new(output))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
