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
from dokime._wire import _pool
# @@protoc_insertion_point(imports)
from dokime._wire.dokime.v1.ExecutionMessages.AppendPlannedCases import messages_pb2 as dokime_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_AppendPlannedCases_dot_messages__pb2
from dokime._wire.dokime.v1.ExecutionMessages.CopyCycle import messages_pb2 as dokime_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_CopyCycle_dot_messages__pb2
from dokime._wire.dokime.v1.ExecutionMessages.CreateCycle import messages_pb2 as dokime_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_CreateCycle_dot_messages__pb2
from dokime._wire.dokime.v1.ExecutionMessages.DeleteCase import messages_pb2 as dokime_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_DeleteCase_dot_messages__pb2
from dokime._wire.dokime.v1.ExecutionMessages.GetCase import messages_pb2 as dokime_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_GetCase_dot_messages__pb2
from dokime._wire.dokime.v1.ExecutionMessages.GetCycle import messages_pb2 as dokime_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_GetCycle_dot_messages__pb2
from dokime._wire.dokime.v1.ExecutionMessages.GetExecution import messages_pb2 as dokime_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_GetExecution_dot_messages__pb2
from dokime._wire.dokime.v1.ExecutionMessages.GetExecutionCommand import messages_pb2 as dokime_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_GetExecutionCommand_dot_messages__pb2
from dokime._wire.dokime.v1.ExecutionMessages.GetRunRequest import messages_pb2 as dokime_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_GetRunRequest_dot_messages__pb2
from dokime._wire.dokime.v1.ExecutionMessages.ListCaseParameters import messages_pb2 as dokime_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_ListCaseParameters_dot_messages__pb2
from dokime._wire.dokime.v1.ExecutionMessages.ListCases import messages_pb2 as dokime_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_ListCases_dot_messages__pb2
from dokime._wire.dokime.v1.ExecutionMessages.ListCycles import messages_pb2 as dokime_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_ListCycles_dot_messages__pb2
from dokime._wire.dokime.v1.ExecutionMessages.ListExecutionParameters import messages_pb2 as dokime_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_ListExecutionParameters_dot_messages__pb2
from dokime._wire.dokime.v1.ExecutionMessages.ListExecutionSteps import messages_pb2 as dokime_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_ListExecutionSteps_dot_messages__pb2
from dokime._wire.dokime.v1.ExecutionMessages.ListExecutions import messages_pb2 as dokime_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_ListExecutions_dot_messages__pb2
from dokime._wire.dokime.v1.ExecutionMessages.ListStepCriteria import messages_pb2 as dokime_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_ListStepCriteria_dot_messages__pb2
from dokime._wire.dokime.v1.ExecutionMessages.PauseCycle import messages_pb2 as dokime_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_PauseCycle_dot_messages__pb2
from dokime._wire.dokime.v1.ExecutionMessages.PauseExecution import messages_pb2 as dokime_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_PauseExecution_dot_messages__pb2
from dokime._wire.dokime.v1.ExecutionMessages.ResumeCycle import messages_pb2 as dokime_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_ResumeCycle_dot_messages__pb2
from dokime._wire.dokime.v1.ExecutionMessages.ResumeExecution import messages_pb2 as dokime_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_ResumeExecution_dot_messages__pb2
from dokime._wire.dokime.v1.ExecutionMessages.RunCase import messages_pb2 as dokime_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_RunCase_dot_messages__pb2
from dokime._wire.dokime.v1.ExecutionMessages.RunCycle import messages_pb2 as dokime_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_RunCycle_dot_messages__pb2
from dokime._wire.dokime.v1.ExecutionMessages.RunExecution import messages_pb2 as dokime_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_RunExecution_dot_messages__pb2
from dokime._wire.dokime.v1.ExecutionMessages.ScheduleCaseRerun import messages_pb2 as dokime_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_ScheduleCaseRerun_dot_messages__pb2
from dokime._wire.dokime.v1.ExecutionMessages.SetCycleArchived import messages_pb2 as dokime_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_SetCycleArchived_dot_messages__pb2
from dokime._wire.dokime.v1.ExecutionMessages.SetExecutionArchived import messages_pb2 as dokime_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_SetExecutionArchived_dot_messages__pb2
from dokime._wire.dokime.v1.ExecutionMessages.StopCycle import messages_pb2 as dokime_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_StopCycle_dot_messages__pb2
from dokime._wire.dokime.v1.ExecutionMessages.StopExecution import messages_pb2 as dokime_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_StopExecution_dot_messages__pb2
from dokime._wire.dokime.v1.ExecutionMessages.SubmitExecutionCommand import messages_pb2 as dokime_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_SubmitExecutionCommand_dot_messages__pb2
from dokime._wire.dokime.v1.ExecutionMessages.UpdateCyclePolicy import messages_pb2 as dokime_dot___wire_dot_dokime_dot_v1_dot_ExecutionMessages_dot_UpdateCyclePolicy_dot_messages__pb2

# Type Checking Imports


# === Public Exports === #

# === Constants === #
# Generated by the protocol buffer compiler.  DO NOT EDIT!
# NO CHECKED-IN PROTOBUF GENCODE
# source: dokime/_wire/dokime/v1/Execution.proto
# Protobuf Python Version: 7.36.2
_runtime_version.ValidateProtobufRuntimeVersion(
    _runtime_version.Domain.PUBLIC,
    7,
    36,
    2,
    '',
    'dokime/_wire/dokime/v1/Execution.proto'
)





DESCRIPTOR = _pool.POOL.AddSerializedFile(b'\n&dokime/_wire/dokime/v1/Execution.proto\x12\tdokime.v1\x1aJdokime/_wire/dokime/v1/ExecutionMessages/AppendPlannedCases/messages.proto\x1a\x41\x64okime/_wire/dokime/v1/ExecutionMessages/CopyCycle/messages.proto\x1a\x43\x64okime/_wire/dokime/v1/ExecutionMessages/CreateCycle/messages.proto\x1a\x42\x64okime/_wire/dokime/v1/ExecutionMessages/DeleteCase/messages.proto\x1a?dokime/_wire/dokime/v1/ExecutionMessages/GetCase/messages.proto\x1a@dokime/_wire/dokime/v1/ExecutionMessages/GetCycle/messages.proto\x1a\x44\x64okime/_wire/dokime/v1/ExecutionMessages/GetExecution/messages.proto\x1aKdokime/_wire/dokime/v1/ExecutionMessages/GetExecutionCommand/messages.proto\x1a\x45\x64okime/_wire/dokime/v1/ExecutionMessages/GetRunRequest/messages.proto\x1aJdokime/_wire/dokime/v1/ExecutionMessages/ListCaseParameters/messages.proto\x1a\x41\x64okime/_wire/dokime/v1/ExecutionMessages/ListCases/messages.proto\x1a\x42\x64okime/_wire/dokime/v1/ExecutionMessages/ListCycles/messages.proto\x1aOdokime/_wire/dokime/v1/ExecutionMessages/ListExecutionParameters/messages.proto\x1aJdokime/_wire/dokime/v1/ExecutionMessages/ListExecutionSteps/messages.proto\x1a\x46\x64okime/_wire/dokime/v1/ExecutionMessages/ListExecutions/messages.proto\x1aHdokime/_wire/dokime/v1/ExecutionMessages/ListStepCriteria/messages.proto\x1a\x42\x64okime/_wire/dokime/v1/ExecutionMessages/PauseCycle/messages.proto\x1a\x46\x64okime/_wire/dokime/v1/ExecutionMessages/PauseExecution/messages.proto\x1a\x43\x64okime/_wire/dokime/v1/ExecutionMessages/ResumeCycle/messages.proto\x1aGdokime/_wire/dokime/v1/ExecutionMessages/ResumeExecution/messages.proto\x1a?dokime/_wire/dokime/v1/ExecutionMessages/RunCase/messages.proto\x1a@dokime/_wire/dokime/v1/ExecutionMessages/RunCycle/messages.proto\x1a\x44\x64okime/_wire/dokime/v1/ExecutionMessages/RunExecution/messages.proto\x1aIdokime/_wire/dokime/v1/ExecutionMessages/ScheduleCaseRerun/messages.proto\x1aHdokime/_wire/dokime/v1/ExecutionMessages/SetCycleArchived/messages.proto\x1aLdokime/_wire/dokime/v1/ExecutionMessages/SetExecutionArchived/messages.proto\x1a\x41\x64okime/_wire/dokime/v1/ExecutionMessages/StopCycle/messages.proto\x1a\x45\x64okime/_wire/dokime/v1/ExecutionMessages/StopExecution/messages.proto\x1aNdokime/_wire/dokime/v1/ExecutionMessages/SubmitExecutionCommand/messages.proto\x1aIdokime/_wire/dokime/v1/ExecutionMessages/UpdateCyclePolicy/messages.proto2\xbc\x1b\n\tExecution\x12\x80\x01\n\x12\x41ppendPlannedCases\x12\x34.dokime.v1.ExecutionMessages.AppendPlannedCases.Row1\x1a\x34.dokime.v1.ExecutionMessages.AppendPlannedCases.Row2\x12\x65\n\tCopyCycle\x12+.dokime.v1.ExecutionMessages.CopyCycle.Row1\x1a+.dokime.v1.ExecutionMessages.CopyCycle.Row2\x12k\n\x0b\x43reateCycle\x12-.dokime.v1.ExecutionMessages.CreateCycle.Row1\x1a-.dokime.v1.ExecutionMessages.CreateCycle.Row2\x12h\n\nDeleteCase\x12,.dokime.v1.ExecutionMessages.DeleteCase.Row1\x1a,.dokime.v1.ExecutionMessages.DeleteCase.Row2\x12_\n\x07GetCase\x12).dokime.v1.ExecutionMessages.GetCase.Row1\x1a).dokime.v1.ExecutionMessages.GetCase.Row2\x12\x62\n\x08GetCycle\x12*.dokime.v1.ExecutionMessages.GetCycle.Row1\x1a*.dokime.v1.ExecutionMessages.GetCycle.Row2\x12n\n\x0cGetExecution\x12..dokime.v1.ExecutionMessages.GetExecution.Row1\x1a..dokime.v1.ExecutionMessages.GetExecution.Row2\x12\x83\x01\n\x13GetExecutionCommand\x12\x35.dokime.v1.ExecutionMessages.GetExecutionCommand.Row1\x1a\x35.dokime.v1.ExecutionMessages.GetExecutionCommand.Row2\x12q\n\rGetRunRequest\x12/.dokime.v1.ExecutionMessages.GetRunRequest.Row1\x1a/.dokime.v1.ExecutionMessages.GetRunRequest.Row2\x12\x80\x01\n\x12ListCaseParameters\x12\x34.dokime.v1.ExecutionMessages.ListCaseParameters.Row1\x1a\x34.dokime.v1.ExecutionMessages.ListCaseParameters.Row2\x12\x65\n\tListCases\x12+.dokime.v1.ExecutionMessages.ListCases.Row1\x1a+.dokime.v1.ExecutionMessages.ListCases.Row2\x12h\n\nListCycles\x12,.dokime.v1.ExecutionMessages.ListCycles.Row1\x1a,.dokime.v1.ExecutionMessages.ListCycles.Row2\x12\x8f\x01\n\x17ListExecutionParameters\x12\x39.dokime.v1.ExecutionMessages.ListExecutionParameters.Row1\x1a\x39.dokime.v1.ExecutionMessages.ListExecutionParameters.Row2\x12\x80\x01\n\x12ListExecutionSteps\x12\x34.dokime.v1.ExecutionMessages.ListExecutionSteps.Row1\x1a\x34.dokime.v1.ExecutionMessages.ListExecutionSteps.Row2\x12t\n\x0eListExecutions\x12\x30.dokime.v1.ExecutionMessages.ListExecutions.Row1\x1a\x30.dokime.v1.ExecutionMessages.ListExecutions.Row2\x12z\n\x10ListStepCriteria\x12\x32.dokime.v1.ExecutionMessages.ListStepCriteria.Row1\x1a\x32.dokime.v1.ExecutionMessages.ListStepCriteria.Row2\x12h\n\nPauseCycle\x12,.dokime.v1.ExecutionMessages.PauseCycle.Row1\x1a,.dokime.v1.ExecutionMessages.PauseCycle.Row2\x12t\n\x0ePauseExecution\x12\x30.dokime.v1.ExecutionMessages.PauseExecution.Row1\x1a\x30.dokime.v1.ExecutionMessages.PauseExecution.Row2\x12k\n\x0bResumeCycle\x12-.dokime.v1.ExecutionMessages.ResumeCycle.Row1\x1a-.dokime.v1.ExecutionMessages.ResumeCycle.Row2\x12w\n\x0fResumeExecution\x12\x31.dokime.v1.ExecutionMessages.ResumeExecution.Row1\x1a\x31.dokime.v1.ExecutionMessages.ResumeExecution.Row2\x12_\n\x07RunCase\x12).dokime.v1.ExecutionMessages.RunCase.Row1\x1a).dokime.v1.ExecutionMessages.RunCase.Row2\x12\x62\n\x08RunCycle\x12*.dokime.v1.ExecutionMessages.RunCycle.Row1\x1a*.dokime.v1.ExecutionMessages.RunCycle.Row2\x12n\n\x0cRunExecution\x12..dokime.v1.ExecutionMessages.RunExecution.Row1\x1a..dokime.v1.ExecutionMessages.RunExecution.Row2\x12}\n\x11ScheduleCaseRerun\x12\x33.dokime.v1.ExecutionMessages.ScheduleCaseRerun.Row1\x1a\x33.dokime.v1.ExecutionMessages.ScheduleCaseRerun.Row2\x12z\n\x10SetCycleArchived\x12\x32.dokime.v1.ExecutionMessages.SetCycleArchived.Row1\x1a\x32.dokime.v1.ExecutionMessages.SetCycleArchived.Row2\x12\x86\x01\n\x14SetExecutionArchived\x12\x36.dokime.v1.ExecutionMessages.SetExecutionArchived.Row1\x1a\x36.dokime.v1.ExecutionMessages.SetExecutionArchived.Row2\x12\x65\n\tStopCycle\x12+.dokime.v1.ExecutionMessages.StopCycle.Row1\x1a+.dokime.v1.ExecutionMessages.StopCycle.Row2\x12q\n\rStopExecution\x12/.dokime.v1.ExecutionMessages.StopExecution.Row1\x1a/.dokime.v1.ExecutionMessages.StopExecution.Row2\x12\x8c\x01\n\x16SubmitExecutionCommand\x12\x38.dokime.v1.ExecutionMessages.SubmitExecutionCommand.Row1\x1a\x38.dokime.v1.ExecutionMessages.SubmitExecutionCommand.Row2\x12}\n\x11UpdateCyclePolicy\x12\x33.dokime.v1.ExecutionMessages.UpdateCyclePolicy.Row1\x1a\x33.dokime.v1.ExecutionMessages.UpdateCyclePolicy.Row2b\x06proto3')

_globals = globals()
_builder.BuildMessageAndEnumDescriptors(DESCRIPTOR, _globals)
_pool.build(DESCRIPTOR, 'dokime._wire.dokime.v1.Execution_pb2', _globals)
if not _descriptor._USE_C_DESCRIPTORS:
  DESCRIPTOR._loaded_options = None
  _globals['_EXECUTION']._serialized_start=2200
  _globals['_EXECUTION']._serialized_end=5716
# @@protoc_insertion_point(module_scope)

# === Types === #

# === Classes === #

# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
