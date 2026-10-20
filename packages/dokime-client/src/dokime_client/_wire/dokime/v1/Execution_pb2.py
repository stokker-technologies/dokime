# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***


"""Generated protocol buffer code."""

# === Imports === #

# Standard Library Imports

# Third Party Imports
from google.protobuf import descriptor as _descriptor
from google.protobuf import runtime_version as _runtime_version
from google.protobuf.internal import builder as _builder

# Local Imports
from dokime_client._wire import _pool
# @@protoc_insertion_point(imports)
from dokime_client._wire.dokime.v1.ExecutionMessages.AppendExecutions import messages_pb2 as dokime__client_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_AppendExecutions_dot_messages__pb2
from dokime_client._wire.dokime.v1.ExecutionMessages.CancelExecution import messages_pb2 as dokime__client_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_CancelExecution_dot_messages__pb2
from dokime_client._wire.dokime.v1.ExecutionMessages.CopyCycle import messages_pb2 as dokime__client_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_CopyCycle_dot_messages__pb2
from dokime_client._wire.dokime.v1.ExecutionMessages.CreateCycle import messages_pb2 as dokime__client_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_CreateCycle_dot_messages__pb2
from dokime_client._wire.dokime.v1.ExecutionMessages.DeleteCase import messages_pb2 as dokime__client_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_DeleteCase_dot_messages__pb2
from dokime_client._wire.dokime.v1.ExecutionMessages.GetCase import messages_pb2 as dokime__client_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_GetCase_dot_messages__pb2
from dokime_client._wire.dokime.v1.ExecutionMessages.GetCycle import messages_pb2 as dokime__client_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_GetCycle_dot_messages__pb2
from dokime_client._wire.dokime.v1.ExecutionMessages.GetExecution import messages_pb2 as dokime__client_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_GetExecution_dot_messages__pb2
from dokime_client._wire.dokime.v1.ExecutionMessages.GetExecutionCommand import messages_pb2 as dokime__client_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_GetExecutionCommand_dot_messages__pb2
from dokime_client._wire.dokime.v1.ExecutionMessages.ListCaseParameters import messages_pb2 as dokime__client_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_ListCaseParameters_dot_messages__pb2
from dokime_client._wire.dokime.v1.ExecutionMessages.ListCases import messages_pb2 as dokime__client_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_ListCases_dot_messages__pb2
from dokime_client._wire.dokime.v1.ExecutionMessages.ListCycles import messages_pb2 as dokime__client_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_ListCycles_dot_messages__pb2
from dokime_client._wire.dokime.v1.ExecutionMessages.ListExecutionParameters import messages_pb2 as dokime__client_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_ListExecutionParameters_dot_messages__pb2
from dokime_client._wire.dokime.v1.ExecutionMessages.ListExecutionSteps import messages_pb2 as dokime__client_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_ListExecutionSteps_dot_messages__pb2
from dokime_client._wire.dokime.v1.ExecutionMessages.ListExecutions import messages_pb2 as dokime__client_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_ListExecutions_dot_messages__pb2
from dokime_client._wire.dokime.v1.ExecutionMessages.ListStepCriteria import messages_pb2 as dokime__client_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_ListStepCriteria_dot_messages__pb2
from dokime_client._wire.dokime.v1.ExecutionMessages.PauseCycle import messages_pb2 as dokime__client_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_PauseCycle_dot_messages__pb2
from dokime_client._wire.dokime.v1.ExecutionMessages.PauseExecution import messages_pb2 as dokime__client_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_PauseExecution_dot_messages__pb2
from dokime_client._wire.dokime.v1.ExecutionMessages.RenameCycle import messages_pb2 as dokime__client_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_RenameCycle_dot_messages__pb2
from dokime_client._wire.dokime.v1.ExecutionMessages.RenameExecution import messages_pb2 as dokime__client_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_RenameExecution_dot_messages__pb2
from dokime_client._wire.dokime.v1.ExecutionMessages.ResumeCycle import messages_pb2 as dokime__client_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_ResumeCycle_dot_messages__pb2
from dokime_client._wire.dokime.v1.ExecutionMessages.ResumeExecution import messages_pb2 as dokime__client_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_ResumeExecution_dot_messages__pb2
from dokime_client._wire.dokime.v1.ExecutionMessages.SealCycle import messages_pb2 as dokime__client_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_SealCycle_dot_messages__pb2
from dokime_client._wire.dokime.v1.ExecutionMessages.SetCycleArchived import messages_pb2 as dokime__client_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_SetCycleArchived_dot_messages__pb2
from dokime_client._wire.dokime.v1.ExecutionMessages.SetExecutionArchived import messages_pb2 as dokime__client_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_SetExecutionArchived_dot_messages__pb2
from dokime_client._wire.dokime.v1.ExecutionMessages.StartExecution import messages_pb2 as dokime__client_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_StartExecution_dot_messages__pb2
from dokime_client._wire.dokime.v1.ExecutionMessages.StopExecution import messages_pb2 as dokime__client_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_StopExecution_dot_messages__pb2
from dokime_client._wire.dokime.v1.ExecutionMessages.SubmitExecutionCommand import messages_pb2 as dokime__client_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_SubmitExecutionCommand_dot_messages__pb2
from dokime_client._wire.dokime.v1.ExecutionMessages.UpdateCyclePolicy import messages_pb2 as dokime__client_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_UpdateCyclePolicy_dot_messages__pb2

# Type Checking Imports


# === Public Exports === #

# === Constants === #
# Generated by the protocol buffer compiler.  DO NOT EDIT!
# NO CHECKED-IN PROTOBUF GENCODE
# source: dokime_client/_wire/dokime/v1/Execution.proto
# Protobuf Python Version: 7.36.2
_runtime_version.ValidateProtobufRuntimeVersion(
    _runtime_version.Domain.PUBLIC,
    7,
    36,
    2,
    '',
    'dokime_client/_wire/dokime/v1/Execution.proto'
)





DESCRIPTOR = _pool.POOL.AddSerializedFile(b'\n-dokime_client/_wire/dokime/v1/Execution.proto\x12\tdokime.v1\x1aOdokime_client/_wire/dokime/v1/ExecutionMessages/AppendExecutions/messages.proto\x1aNdokime_client/_wire/dokime/v1/ExecutionMessages/CancelExecution/messages.proto\x1aHdokime_client/_wire/dokime/v1/ExecutionMessages/CopyCycle/messages.proto\x1aJdokime_client/_wire/dokime/v1/ExecutionMessages/CreateCycle/messages.proto\x1aIdokime_client/_wire/dokime/v1/ExecutionMessages/DeleteCase/messages.proto\x1a\x46\x64okime_client/_wire/dokime/v1/ExecutionMessages/GetCase/messages.proto\x1aGdokime_client/_wire/dokime/v1/ExecutionMessages/GetCycle/messages.proto\x1aKdokime_client/_wire/dokime/v1/ExecutionMessages/GetExecution/messages.proto\x1aRdokime_client/_wire/dokime/v1/ExecutionMessages/GetExecutionCommand/messages.proto\x1aQdokime_client/_wire/dokime/v1/ExecutionMessages/ListCaseParameters/messages.proto\x1aHdokime_client/_wire/dokime/v1/ExecutionMessages/ListCases/messages.proto\x1aIdokime_client/_wire/dokime/v1/ExecutionMessages/ListCycles/messages.proto\x1aVdokime_client/_wire/dokime/v1/ExecutionMessages/ListExecutionParameters/messages.proto\x1aQdokime_client/_wire/dokime/v1/ExecutionMessages/ListExecutionSteps/messages.proto\x1aMdokime_client/_wire/dokime/v1/ExecutionMessages/ListExecutions/messages.proto\x1aOdokime_client/_wire/dokime/v1/ExecutionMessages/ListStepCriteria/messages.proto\x1aIdokime_client/_wire/dokime/v1/ExecutionMessages/PauseCycle/messages.proto\x1aMdokime_client/_wire/dokime/v1/ExecutionMessages/PauseExecution/messages.proto\x1aJdokime_client/_wire/dokime/v1/ExecutionMessages/RenameCycle/messages.proto\x1aNdokime_client/_wire/dokime/v1/ExecutionMessages/RenameExecution/messages.proto\x1aJdokime_client/_wire/dokime/v1/ExecutionMessages/ResumeCycle/messages.proto\x1aNdokime_client/_wire/dokime/v1/ExecutionMessages/ResumeExecution/messages.proto\x1aHdokime_client/_wire/dokime/v1/ExecutionMessages/SealCycle/messages.proto\x1aOdokime_client/_wire/dokime/v1/ExecutionMessages/SetCycleArchived/messages.proto\x1aSdokime_client/_wire/dokime/v1/ExecutionMessages/SetExecutionArchived/messages.proto\x1aMdokime_client/_wire/dokime/v1/ExecutionMessages/StartExecution/messages.proto\x1aLdokime_client/_wire/dokime/v1/ExecutionMessages/StopExecution/messages.proto\x1aUdokime_client/_wire/dokime/v1/ExecutionMessages/SubmitExecutionCommand/messages.proto\x1aPdokime_client/_wire/dokime/v1/ExecutionMessages/UpdateCyclePolicy/messages.proto2\xe3\x1a\n\tExecution\x12z\n\x10\x41ppendExecutions\x12\x32.dokime.v1.ExecutionMessages.AppendExecutions.Row1\x1a\x32.dokime.v1.ExecutionMessages.AppendExecutions.Row2\x12w\n\x0f\x43\x61ncelExecution\x12\x31.dokime.v1.ExecutionMessages.CancelExecution.Row1\x1a\x31.dokime.v1.ExecutionMessages.CancelExecution.Row2\x12\x65\n\tCopyCycle\x12+.dokime.v1.ExecutionMessages.CopyCycle.Row1\x1a+.dokime.v1.ExecutionMessages.CopyCycle.Row2\x12k\n\x0b\x43reateCycle\x12-.dokime.v1.ExecutionMessages.CreateCycle.Row1\x1a-.dokime.v1.ExecutionMessages.CreateCycle.Row2\x12h\n\nDeleteCase\x12,.dokime.v1.ExecutionMessages.DeleteCase.Row1\x1a,.dokime.v1.ExecutionMessages.DeleteCase.Row2\x12_\n\x07GetCase\x12).dokime.v1.ExecutionMessages.GetCase.Row1\x1a).dokime.v1.ExecutionMessages.GetCase.Row2\x12\x62\n\x08GetCycle\x12*.dokime.v1.ExecutionMessages.GetCycle.Row1\x1a*.dokime.v1.ExecutionMessages.GetCycle.Row2\x12n\n\x0cGetExecution\x12..dokime.v1.ExecutionMessages.GetExecution.Row1\x1a..dokime.v1.ExecutionMessages.GetExecution.Row2\x12\x83\x01\n\x13GetExecutionCommand\x12\x35.dokime.v1.ExecutionMessages.GetExecutionCommand.Row1\x1a\x35.dokime.v1.ExecutionMessages.GetExecutionCommand.Row2\x12\x80\x01\n\x12ListCaseParameters\x12\x34.dokime.v1.ExecutionMessages.ListCaseParameters.Row1\x1a\x34.dokime.v1.ExecutionMessages.ListCaseParameters.Row2\x12\x65\n\tListCases\x12+.dokime.v1.ExecutionMessages.ListCases.Row1\x1a+.dokime.v1.ExecutionMessages.ListCases.Row2\x12h\n\nListCycles\x12,.dokime.v1.ExecutionMessages.ListCycles.Row1\x1a,.dokime.v1.ExecutionMessages.ListCycles.Row2\x12\x8f\x01\n\x17ListExecutionParameters\x12\x39.dokime.v1.ExecutionMessages.ListExecutionParameters.Row1\x1a\x39.dokime.v1.ExecutionMessages.ListExecutionParameters.Row2\x12\x80\x01\n\x12ListExecutionSteps\x12\x34.dokime.v1.ExecutionMessages.ListExecutionSteps.Row1\x1a\x34.dokime.v1.ExecutionMessages.ListExecutionSteps.Row2\x12t\n\x0eListExecutions\x12\x30.dokime.v1.ExecutionMessages.ListExecutions.Row1\x1a\x30.dokime.v1.ExecutionMessages.ListExecutions.Row2\x12z\n\x10ListStepCriteria\x12\x32.dokime.v1.ExecutionMessages.ListStepCriteria.Row1\x1a\x32.dokime.v1.ExecutionMessages.ListStepCriteria.Row2\x12h\n\nPauseCycle\x12,.dokime.v1.ExecutionMessages.PauseCycle.Row1\x1a,.dokime.v1.ExecutionMessages.PauseCycle.Row2\x12t\n\x0ePauseExecution\x12\x30.dokime.v1.ExecutionMessages.PauseExecution.Row1\x1a\x30.dokime.v1.ExecutionMessages.PauseExecution.Row2\x12k\n\x0bRenameCycle\x12-.dokime.v1.ExecutionMessages.RenameCycle.Row1\x1a-.dokime.v1.ExecutionMessages.RenameCycle.Row2\x12w\n\x0fRenameExecution\x12\x31.dokime.v1.ExecutionMessages.RenameExecution.Row1\x1a\x31.dokime.v1.ExecutionMessages.RenameExecution.Row2\x12k\n\x0bResumeCycle\x12-.dokime.v1.ExecutionMessages.ResumeCycle.Row1\x1a-.dokime.v1.ExecutionMessages.ResumeCycle.Row2\x12w\n\x0fResumeExecution\x12\x31.dokime.v1.ExecutionMessages.ResumeExecution.Row1\x1a\x31.dokime.v1.ExecutionMessages.ResumeExecution.Row2\x12\x65\n\tSealCycle\x12+.dokime.v1.ExecutionMessages.SealCycle.Row1\x1a+.dokime.v1.ExecutionMessages.SealCycle.Row2\x12z\n\x10SetCycleArchived\x12\x32.dokime.v1.ExecutionMessages.SetCycleArchived.Row1\x1a\x32.dokime.v1.ExecutionMessages.SetCycleArchived.Row2\x12\x86\x01\n\x14SetExecutionArchived\x12\x36.dokime.v1.ExecutionMessages.SetExecutionArchived.Row1\x1a\x36.dokime.v1.ExecutionMessages.SetExecutionArchived.Row2\x12t\n\x0eStartExecution\x12\x30.dokime.v1.ExecutionMessages.StartExecution.Row1\x1a\x30.dokime.v1.ExecutionMessages.StartExecution.Row2\x12q\n\rStopExecution\x12/.dokime.v1.ExecutionMessages.StopExecution.Row1\x1a/.dokime.v1.ExecutionMessages.StopExecution.Row2\x12\x8c\x01\n\x16SubmitExecutionCommand\x12\x38.dokime.v1.ExecutionMessages.SubmitExecutionCommand.Row1\x1a\x38.dokime.v1.ExecutionMessages.SubmitExecutionCommand.Row2\x12}\n\x11UpdateCyclePolicy\x12\x33.dokime.v1.ExecutionMessages.UpdateCyclePolicy.Row1\x1a\x33.dokime.v1.ExecutionMessages.UpdateCyclePolicy.Row2b\x06proto3')

_globals = globals()
_builder.BuildMessageAndEnumDescriptors(DESCRIPTOR, _globals)
_pool.build(DESCRIPTOR, 'dokime_client._wire.dokime.v1.Execution_pb2', _globals)
if not _descriptor._USE_C_DESCRIPTORS:
  DESCRIPTOR._loaded_options = None
  _globals['_EXECUTION']._serialized_start=2348
  _globals['_EXECUTION']._serialized_end=5775
# @@protoc_insertion_point(module_scope)

# === Types === #

# === Classes === #

# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
