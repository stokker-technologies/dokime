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





DESCRIPTOR = _pool.POOL.AddSerializedFile(b'\n-dokime_client/_wire/dokime/v1/Execution.proto\x12\tdokime.v1\x1aOdokime_client/_wire/dokime/v1/ExecutionMessages/AppendExecutions/messages.proto\x1aNdokime_client/_wire/dokime/v1/ExecutionMessages/CancelExecution/messages.proto\x1aHdokime_client/_wire/dokime/v1/ExecutionMessages/CopyCycle/messages.proto\x1aJdokime_client/_wire/dokime/v1/ExecutionMessages/CreateCycle/messages.proto\x1aIdokime_client/_wire/dokime/v1/ExecutionMessages/DeleteCase/messages.proto\x1a\x46\x64okime_client/_wire/dokime/v1/ExecutionMessages/GetCase/messages.proto\x1aGdokime_client/_wire/dokime/v1/ExecutionMessages/GetCycle/messages.proto\x1aKdokime_client/_wire/dokime/v1/ExecutionMessages/GetExecution/messages.proto\x1aRdokime_client/_wire/dokime/v1/ExecutionMessages/GetExecutionCommand/messages.proto\x1aQdokime_client/_wire/dokime/v1/ExecutionMessages/ListCaseParameters/messages.proto\x1aHdokime_client/_wire/dokime/v1/ExecutionMessages/ListCases/messages.proto\x1aIdokime_client/_wire/dokime/v1/ExecutionMessages/ListCycles/messages.proto\x1aVdokime_client/_wire/dokime/v1/ExecutionMessages/ListExecutionParameters/messages.proto\x1aQdokime_client/_wire/dokime/v1/ExecutionMessages/ListExecutionSteps/messages.proto\x1aMdokime_client/_wire/dokime/v1/ExecutionMessages/ListExecutions/messages.proto\x1aIdokime_client/_wire/dokime/v1/ExecutionMessages/PauseCycle/messages.proto\x1aMdokime_client/_wire/dokime/v1/ExecutionMessages/PauseExecution/messages.proto\x1aJdokime_client/_wire/dokime/v1/ExecutionMessages/RenameCycle/messages.proto\x1aNdokime_client/_wire/dokime/v1/ExecutionMessages/RenameExecution/messages.proto\x1aJdokime_client/_wire/dokime/v1/ExecutionMessages/ResumeCycle/messages.proto\x1aNdokime_client/_wire/dokime/v1/ExecutionMessages/ResumeExecution/messages.proto\x1aHdokime_client/_wire/dokime/v1/ExecutionMessages/SealCycle/messages.proto\x1aOdokime_client/_wire/dokime/v1/ExecutionMessages/SetCycleArchived/messages.proto\x1aSdokime_client/_wire/dokime/v1/ExecutionMessages/SetExecutionArchived/messages.proto\x1aMdokime_client/_wire/dokime/v1/ExecutionMessages/StartExecution/messages.proto\x1aLdokime_client/_wire/dokime/v1/ExecutionMessages/StopExecution/messages.proto\x1aUdokime_client/_wire/dokime/v1/ExecutionMessages/SubmitExecutionCommand/messages.proto\x1aPdokime_client/_wire/dokime/v1/ExecutionMessages/UpdateCyclePolicy/messages.proto\"\'\n%ExecutionAppendExecutionsCancellation\"\x94\x02\n!ExecutionAppendExecutionsRunInput\x12N\n\x07request\x18\x01 \x01(\x0b\x32\x32.dokime.v1.ExecutionMessages.AppendExecutions.Row1H\x00R\x07request\x12J\n\x06\x63\x61ncel\x18\x03 \x01(\x0b\x32\x30.dokime.v1.ExecutionAppendExecutionsCancellationH\x00R\x06\x63\x61ncel\x12J\n\x06\x66inish\x18\x04 \x01(\x0b\x32\x30.dokime.v1.ExecutionAppendExecutionsCancellationH\x00R\x06\x66inishB\x07\n\x05value\"\x7f\n\"ExecutionAppendExecutionsRunOutput\x12P\n\x08response\x18\x02 \x01(\x0b\x32\x32.dokime.v1.ExecutionMessages.AppendExecutions.Row2H\x00R\x08responseB\x07\n\x05value\"&\n$ExecutionCancelExecutionCancellation\"\x90\x02\n ExecutionCancelExecutionRunInput\x12M\n\x07request\x18\x01 \x01(\x0b\x32\x31.dokime.v1.ExecutionMessages.CancelExecution.Row1H\x00R\x07request\x12I\n\x06\x63\x61ncel\x18\x03 \x01(\x0b\x32/.dokime.v1.ExecutionCancelExecutionCancellationH\x00R\x06\x63\x61ncel\x12I\n\x06\x66inish\x18\x04 \x01(\x0b\x32/.dokime.v1.ExecutionCancelExecutionCancellationH\x00R\x06\x66inishB\x07\n\x05value\"}\n!ExecutionCancelExecutionRunOutput\x12O\n\x08response\x18\x02 \x01(\x0b\x32\x31.dokime.v1.ExecutionMessages.CancelExecution.Row2H\x00R\x08responseB\x07\n\x05value\" \n\x1e\x45xecutionCopyCycleCancellation\"\xf8\x01\n\x1a\x45xecutionCopyCycleRunInput\x12G\n\x07request\x18\x01 \x01(\x0b\x32+.dokime.v1.ExecutionMessages.CopyCycle.Row1H\x00R\x07request\x12\x43\n\x06\x63\x61ncel\x18\x03 \x01(\x0b\x32).dokime.v1.ExecutionCopyCycleCancellationH\x00R\x06\x63\x61ncel\x12\x43\n\x06\x66inish\x18\x04 \x01(\x0b\x32).dokime.v1.ExecutionCopyCycleCancellationH\x00R\x06\x66inishB\x07\n\x05value\"q\n\x1b\x45xecutionCopyCycleRunOutput\x12I\n\x08response\x18\x02 \x01(\x0b\x32+.dokime.v1.ExecutionMessages.CopyCycle.Row2H\x00R\x08responseB\x07\n\x05value\"\"\n ExecutionCreateCycleCancellation\"\x80\x02\n\x1c\x45xecutionCreateCycleRunInput\x12I\n\x07request\x18\x01 \x01(\x0b\x32-.dokime.v1.ExecutionMessages.CreateCycle.Row1H\x00R\x07request\x12\x45\n\x06\x63\x61ncel\x18\x03 \x01(\x0b\x32+.dokime.v1.ExecutionCreateCycleCancellationH\x00R\x06\x63\x61ncel\x12\x45\n\x06\x66inish\x18\x04 \x01(\x0b\x32+.dokime.v1.ExecutionCreateCycleCancellationH\x00R\x06\x66inishB\x07\n\x05value\"u\n\x1d\x45xecutionCreateCycleRunOutput\x12K\n\x08response\x18\x02 \x01(\x0b\x32-.dokime.v1.ExecutionMessages.CreateCycle.Row2H\x00R\x08responseB\x07\n\x05value\"!\n\x1f\x45xecutionDeleteCaseCancellation\"\xfc\x01\n\x1b\x45xecutionDeleteCaseRunInput\x12H\n\x07request\x18\x01 \x01(\x0b\x32,.dokime.v1.ExecutionMessages.DeleteCase.Row1H\x00R\x07request\x12\x44\n\x06\x63\x61ncel\x18\x03 \x01(\x0b\x32*.dokime.v1.ExecutionDeleteCaseCancellationH\x00R\x06\x63\x61ncel\x12\x44\n\x06\x66inish\x18\x04 \x01(\x0b\x32*.dokime.v1.ExecutionDeleteCaseCancellationH\x00R\x06\x66inishB\x07\n\x05value\"s\n\x1c\x45xecutionDeleteCaseRunOutput\x12J\n\x08response\x18\x02 \x01(\x0b\x32,.dokime.v1.ExecutionMessages.DeleteCase.Row2H\x00R\x08responseB\x07\n\x05value\"\x1e\n\x1c\x45xecutionGetCaseCancellation\"\xf0\x01\n\x18\x45xecutionGetCaseRunInput\x12\x45\n\x07request\x18\x01 \x01(\x0b\x32).dokime.v1.ExecutionMessages.GetCase.Row1H\x00R\x07request\x12\x41\n\x06\x63\x61ncel\x18\x03 \x01(\x0b\x32\'.dokime.v1.ExecutionGetCaseCancellationH\x00R\x06\x63\x61ncel\x12\x41\n\x06\x66inish\x18\x04 \x01(\x0b\x32\'.dokime.v1.ExecutionGetCaseCancellationH\x00R\x06\x66inishB\x07\n\x05value\"m\n\x19\x45xecutionGetCaseRunOutput\x12G\n\x08response\x18\x02 \x01(\x0b\x32).dokime.v1.ExecutionMessages.GetCase.Row2H\x00R\x08responseB\x07\n\x05value\"\x1f\n\x1d\x45xecutionGetCycleCancellation\"\xf4\x01\n\x19\x45xecutionGetCycleRunInput\x12\x46\n\x07request\x18\x01 \x01(\x0b\x32*.dokime.v1.ExecutionMessages.GetCycle.Row1H\x00R\x07request\x12\x42\n\x06\x63\x61ncel\x18\x03 \x01(\x0b\x32(.dokime.v1.ExecutionGetCycleCancellationH\x00R\x06\x63\x61ncel\x12\x42\n\x06\x66inish\x18\x04 \x01(\x0b\x32(.dokime.v1.ExecutionGetCycleCancellationH\x00R\x06\x66inishB\x07\n\x05value\"o\n\x1a\x45xecutionGetCycleRunOutput\x12H\n\x08response\x18\x02 \x01(\x0b\x32*.dokime.v1.ExecutionMessages.GetCycle.Row2H\x00R\x08responseB\x07\n\x05value\"#\n!ExecutionGetExecutionCancellation\"\x84\x02\n\x1d\x45xecutionGetExecutionRunInput\x12J\n\x07request\x18\x01 \x01(\x0b\x32..dokime.v1.ExecutionMessages.GetExecution.Row1H\x00R\x07request\x12\x46\n\x06\x63\x61ncel\x18\x03 \x01(\x0b\x32,.dokime.v1.ExecutionGetExecutionCancellationH\x00R\x06\x63\x61ncel\x12\x46\n\x06\x66inish\x18\x04 \x01(\x0b\x32,.dokime.v1.ExecutionGetExecutionCancellationH\x00R\x06\x66inishB\x07\n\x05value\"w\n\x1e\x45xecutionGetExecutionRunOutput\x12L\n\x08response\x18\x02 \x01(\x0b\x32..dokime.v1.ExecutionMessages.GetExecution.Row2H\x00R\x08responseB\x07\n\x05value\"*\n(ExecutionGetExecutionCommandCancellation\"\xa0\x02\n$ExecutionGetExecutionCommandRunInput\x12Q\n\x07request\x18\x01 \x01(\x0b\x32\x35.dokime.v1.ExecutionMessages.GetExecutionCommand.Row1H\x00R\x07request\x12M\n\x06\x63\x61ncel\x18\x03 \x01(\x0b\x32\x33.dokime.v1.ExecutionGetExecutionCommandCancellationH\x00R\x06\x63\x61ncel\x12M\n\x06\x66inish\x18\x04 \x01(\x0b\x32\x33.dokime.v1.ExecutionGetExecutionCommandCancellationH\x00R\x06\x66inishB\x07\n\x05value\"\x85\x01\n%ExecutionGetExecutionCommandRunOutput\x12S\n\x08response\x18\x02 \x01(\x0b\x32\x35.dokime.v1.ExecutionMessages.GetExecutionCommand.Row2H\x00R\x08responseB\x07\n\x05value\")\n\'ExecutionListCaseParametersCancellation\"\x9c\x02\n#ExecutionListCaseParametersRunInput\x12P\n\x07request\x18\x01 \x01(\x0b\x32\x34.dokime.v1.ExecutionMessages.ListCaseParameters.Row1H\x00R\x07request\x12L\n\x06\x63\x61ncel\x18\x03 \x01(\x0b\x32\x32.dokime.v1.ExecutionListCaseParametersCancellationH\x00R\x06\x63\x61ncel\x12L\n\x06\x66inish\x18\x04 \x01(\x0b\x32\x32.dokime.v1.ExecutionListCaseParametersCancellationH\x00R\x06\x66inishB\x07\n\x05value\"\x83\x01\n$ExecutionListCaseParametersRunOutput\x12R\n\x08response\x18\x02 \x01(\x0b\x32\x34.dokime.v1.ExecutionMessages.ListCaseParameters.Row2H\x00R\x08responseB\x07\n\x05value\" \n\x1e\x45xecutionListCasesCancellation\"\xf8\x01\n\x1a\x45xecutionListCasesRunInput\x12G\n\x07request\x18\x01 \x01(\x0b\x32+.dokime.v1.ExecutionMessages.ListCases.Row1H\x00R\x07request\x12\x43\n\x06\x63\x61ncel\x18\x03 \x01(\x0b\x32).dokime.v1.ExecutionListCasesCancellationH\x00R\x06\x63\x61ncel\x12\x43\n\x06\x66inish\x18\x04 \x01(\x0b\x32).dokime.v1.ExecutionListCasesCancellationH\x00R\x06\x66inishB\x07\n\x05value\"q\n\x1b\x45xecutionListCasesRunOutput\x12I\n\x08response\x18\x02 \x01(\x0b\x32+.dokime.v1.ExecutionMessages.ListCases.Row2H\x00R\x08responseB\x07\n\x05value\"!\n\x1f\x45xecutionListCyclesCancellation\"\xfc\x01\n\x1b\x45xecutionListCyclesRunInput\x12H\n\x07request\x18\x01 \x01(\x0b\x32,.dokime.v1.ExecutionMessages.ListCycles.Row1H\x00R\x07request\x12\x44\n\x06\x63\x61ncel\x18\x03 \x01(\x0b\x32*.dokime.v1.ExecutionListCyclesCancellationH\x00R\x06\x63\x61ncel\x12\x44\n\x06\x66inish\x18\x04 \x01(\x0b\x32*.dokime.v1.ExecutionListCyclesCancellationH\x00R\x06\x66inishB\x07\n\x05value\"s\n\x1c\x45xecutionListCyclesRunOutput\x12J\n\x08response\x18\x02 \x01(\x0b\x32,.dokime.v1.ExecutionMessages.ListCycles.Row2H\x00R\x08responseB\x07\n\x05value\".\n,ExecutionListExecutionParametersCancellation\"\xb0\x02\n(ExecutionListExecutionParametersRunInput\x12U\n\x07request\x18\x01 \x01(\x0b\x32\x39.dokime.v1.ExecutionMessages.ListExecutionParameters.Row1H\x00R\x07request\x12Q\n\x06\x63\x61ncel\x18\x03 \x01(\x0b\x32\x37.dokime.v1.ExecutionListExecutionParametersCancellationH\x00R\x06\x63\x61ncel\x12Q\n\x06\x66inish\x18\x04 \x01(\x0b\x32\x37.dokime.v1.ExecutionListExecutionParametersCancellationH\x00R\x06\x66inishB\x07\n\x05value\"\x8d\x01\n)ExecutionListExecutionParametersRunOutput\x12W\n\x08response\x18\x02 \x01(\x0b\x32\x39.dokime.v1.ExecutionMessages.ListExecutionParameters.Row2H\x00R\x08responseB\x07\n\x05value\")\n\'ExecutionListExecutionStepsCancellation\"\x9c\x02\n#ExecutionListExecutionStepsRunInput\x12P\n\x07request\x18\x01 \x01(\x0b\x32\x34.dokime.v1.ExecutionMessages.ListExecutionSteps.Row1H\x00R\x07request\x12L\n\x06\x63\x61ncel\x18\x03 \x01(\x0b\x32\x32.dokime.v1.ExecutionListExecutionStepsCancellationH\x00R\x06\x63\x61ncel\x12L\n\x06\x66inish\x18\x04 \x01(\x0b\x32\x32.dokime.v1.ExecutionListExecutionStepsCancellationH\x00R\x06\x66inishB\x07\n\x05value\"\x83\x01\n$ExecutionListExecutionStepsRunOutput\x12R\n\x08response\x18\x02 \x01(\x0b\x32\x34.dokime.v1.ExecutionMessages.ListExecutionSteps.Row2H\x00R\x08responseB\x07\n\x05value\"%\n#ExecutionListExecutionsCancellation\"\x8c\x02\n\x1f\x45xecutionListExecutionsRunInput\x12L\n\x07request\x18\x01 \x01(\x0b\x32\x30.dokime.v1.ExecutionMessages.ListExecutions.Row1H\x00R\x07request\x12H\n\x06\x63\x61ncel\x18\x03 \x01(\x0b\x32..dokime.v1.ExecutionListExecutionsCancellationH\x00R\x06\x63\x61ncel\x12H\n\x06\x66inish\x18\x04 \x01(\x0b\x32..dokime.v1.ExecutionListExecutionsCancellationH\x00R\x06\x66inishB\x07\n\x05value\"{\n ExecutionListExecutionsRunOutput\x12N\n\x08response\x18\x02 \x01(\x0b\x32\x30.dokime.v1.ExecutionMessages.ListExecutions.Row2H\x00R\x08responseB\x07\n\x05value\"!\n\x1f\x45xecutionPauseCycleCancellation\"\xfc\x01\n\x1b\x45xecutionPauseCycleRunInput\x12H\n\x07request\x18\x01 \x01(\x0b\x32,.dokime.v1.ExecutionMessages.PauseCycle.Row1H\x00R\x07request\x12\x44\n\x06\x63\x61ncel\x18\x03 \x01(\x0b\x32*.dokime.v1.ExecutionPauseCycleCancellationH\x00R\x06\x63\x61ncel\x12\x44\n\x06\x66inish\x18\x04 \x01(\x0b\x32*.dokime.v1.ExecutionPauseCycleCancellationH\x00R\x06\x66inishB\x07\n\x05value\"s\n\x1c\x45xecutionPauseCycleRunOutput\x12J\n\x08response\x18\x02 \x01(\x0b\x32,.dokime.v1.ExecutionMessages.PauseCycle.Row2H\x00R\x08responseB\x07\n\x05value\"%\n#ExecutionPauseExecutionCancellation\"\x8c\x02\n\x1f\x45xecutionPauseExecutionRunInput\x12L\n\x07request\x18\x01 \x01(\x0b\x32\x30.dokime.v1.ExecutionMessages.PauseExecution.Row1H\x00R\x07request\x12H\n\x06\x63\x61ncel\x18\x03 \x01(\x0b\x32..dokime.v1.ExecutionPauseExecutionCancellationH\x00R\x06\x63\x61ncel\x12H\n\x06\x66inish\x18\x04 \x01(\x0b\x32..dokime.v1.ExecutionPauseExecutionCancellationH\x00R\x06\x66inishB\x07\n\x05value\"{\n ExecutionPauseExecutionRunOutput\x12N\n\x08response\x18\x02 \x01(\x0b\x32\x30.dokime.v1.ExecutionMessages.PauseExecution.Row2H\x00R\x08responseB\x07\n\x05value\"\"\n ExecutionRenameCycleCancellation\"\x80\x02\n\x1c\x45xecutionRenameCycleRunInput\x12I\n\x07request\x18\x01 \x01(\x0b\x32-.dokime.v1.ExecutionMessages.RenameCycle.Row1H\x00R\x07request\x12\x45\n\x06\x63\x61ncel\x18\x03 \x01(\x0b\x32+.dokime.v1.ExecutionRenameCycleCancellationH\x00R\x06\x63\x61ncel\x12\x45\n\x06\x66inish\x18\x04 \x01(\x0b\x32+.dokime.v1.ExecutionRenameCycleCancellationH\x00R\x06\x66inishB\x07\n\x05value\"u\n\x1d\x45xecutionRenameCycleRunOutput\x12K\n\x08response\x18\x02 \x01(\x0b\x32-.dokime.v1.ExecutionMessages.RenameCycle.Row2H\x00R\x08responseB\x07\n\x05value\"&\n$ExecutionRenameExecutionCancellation\"\x90\x02\n ExecutionRenameExecutionRunInput\x12M\n\x07request\x18\x01 \x01(\x0b\x32\x31.dokime.v1.ExecutionMessages.RenameExecution.Row1H\x00R\x07request\x12I\n\x06\x63\x61ncel\x18\x03 \x01(\x0b\x32/.dokime.v1.ExecutionRenameExecutionCancellationH\x00R\x06\x63\x61ncel\x12I\n\x06\x66inish\x18\x04 \x01(\x0b\x32/.dokime.v1.ExecutionRenameExecutionCancellationH\x00R\x06\x66inishB\x07\n\x05value\"}\n!ExecutionRenameExecutionRunOutput\x12O\n\x08response\x18\x02 \x01(\x0b\x32\x31.dokime.v1.ExecutionMessages.RenameExecution.Row2H\x00R\x08responseB\x07\n\x05value\"\"\n ExecutionResumeCycleCancellation\"\x80\x02\n\x1c\x45xecutionResumeCycleRunInput\x12I\n\x07request\x18\x01 \x01(\x0b\x32-.dokime.v1.ExecutionMessages.ResumeCycle.Row1H\x00R\x07request\x12\x45\n\x06\x63\x61ncel\x18\x03 \x01(\x0b\x32+.dokime.v1.ExecutionResumeCycleCancellationH\x00R\x06\x63\x61ncel\x12\x45\n\x06\x66inish\x18\x04 \x01(\x0b\x32+.dokime.v1.ExecutionResumeCycleCancellationH\x00R\x06\x66inishB\x07\n\x05value\"u\n\x1d\x45xecutionResumeCycleRunOutput\x12K\n\x08response\x18\x02 \x01(\x0b\x32-.dokime.v1.ExecutionMessages.ResumeCycle.Row2H\x00R\x08responseB\x07\n\x05value\"&\n$ExecutionResumeExecutionCancellation\"\x90\x02\n ExecutionResumeExecutionRunInput\x12M\n\x07request\x18\x01 \x01(\x0b\x32\x31.dokime.v1.ExecutionMessages.ResumeExecution.Row1H\x00R\x07request\x12I\n\x06\x63\x61ncel\x18\x03 \x01(\x0b\x32/.dokime.v1.ExecutionResumeExecutionCancellationH\x00R\x06\x63\x61ncel\x12I\n\x06\x66inish\x18\x04 \x01(\x0b\x32/.dokime.v1.ExecutionResumeExecutionCancellationH\x00R\x06\x66inishB\x07\n\x05value\"}\n!ExecutionResumeExecutionRunOutput\x12O\n\x08response\x18\x02 \x01(\x0b\x32\x31.dokime.v1.ExecutionMessages.ResumeExecution.Row2H\x00R\x08responseB\x07\n\x05value\" \n\x1e\x45xecutionSealCycleCancellation\"\xf8\x01\n\x1a\x45xecutionSealCycleRunInput\x12G\n\x07request\x18\x01 \x01(\x0b\x32+.dokime.v1.ExecutionMessages.SealCycle.Row1H\x00R\x07request\x12\x43\n\x06\x63\x61ncel\x18\x03 \x01(\x0b\x32).dokime.v1.ExecutionSealCycleCancellationH\x00R\x06\x63\x61ncel\x12\x43\n\x06\x66inish\x18\x04 \x01(\x0b\x32).dokime.v1.ExecutionSealCycleCancellationH\x00R\x06\x66inishB\x07\n\x05value\"q\n\x1b\x45xecutionSealCycleRunOutput\x12I\n\x08response\x18\x02 \x01(\x0b\x32+.dokime.v1.ExecutionMessages.SealCycle.Row2H\x00R\x08responseB\x07\n\x05value\"\'\n%ExecutionSetCycleArchivedCancellation\"\x94\x02\n!ExecutionSetCycleArchivedRunInput\x12N\n\x07request\x18\x01 \x01(\x0b\x32\x32.dokime.v1.ExecutionMessages.SetCycleArchived.Row1H\x00R\x07request\x12J\n\x06\x63\x61ncel\x18\x03 \x01(\x0b\x32\x30.dokime.v1.ExecutionSetCycleArchivedCancellationH\x00R\x06\x63\x61ncel\x12J\n\x06\x66inish\x18\x04 \x01(\x0b\x32\x30.dokime.v1.ExecutionSetCycleArchivedCancellationH\x00R\x06\x66inishB\x07\n\x05value\"\x7f\n\"ExecutionSetCycleArchivedRunOutput\x12P\n\x08response\x18\x02 \x01(\x0b\x32\x32.dokime.v1.ExecutionMessages.SetCycleArchived.Row2H\x00R\x08responseB\x07\n\x05value\"+\n)ExecutionSetExecutionArchivedCancellation\"\xa4\x02\n%ExecutionSetExecutionArchivedRunInput\x12R\n\x07request\x18\x01 \x01(\x0b\x32\x36.dokime.v1.ExecutionMessages.SetExecutionArchived.Row1H\x00R\x07request\x12N\n\x06\x63\x61ncel\x18\x03 \x01(\x0b\x32\x34.dokime.v1.ExecutionSetExecutionArchivedCancellationH\x00R\x06\x63\x61ncel\x12N\n\x06\x66inish\x18\x04 \x01(\x0b\x32\x34.dokime.v1.ExecutionSetExecutionArchivedCancellationH\x00R\x06\x66inishB\x07\n\x05value\"\x87\x01\n&ExecutionSetExecutionArchivedRunOutput\x12T\n\x08response\x18\x02 \x01(\x0b\x32\x36.dokime.v1.ExecutionMessages.SetExecutionArchived.Row2H\x00R\x08responseB\x07\n\x05value\"%\n#ExecutionStartExecutionCancellation\"\x8c\x02\n\x1f\x45xecutionStartExecutionRunInput\x12L\n\x07request\x18\x01 \x01(\x0b\x32\x30.dokime.v1.ExecutionMessages.StartExecution.Row1H\x00R\x07request\x12H\n\x06\x63\x61ncel\x18\x03 \x01(\x0b\x32..dokime.v1.ExecutionStartExecutionCancellationH\x00R\x06\x63\x61ncel\x12H\n\x06\x66inish\x18\x04 \x01(\x0b\x32..dokime.v1.ExecutionStartExecutionCancellationH\x00R\x06\x66inishB\x07\n\x05value\"{\n ExecutionStartExecutionRunOutput\x12N\n\x08response\x18\x02 \x01(\x0b\x32\x30.dokime.v1.ExecutionMessages.StartExecution.Row2H\x00R\x08responseB\x07\n\x05value\"$\n\"ExecutionStopExecutionCancellation\"\x88\x02\n\x1e\x45xecutionStopExecutionRunInput\x12K\n\x07request\x18\x01 \x01(\x0b\x32/.dokime.v1.ExecutionMessages.StopExecution.Row1H\x00R\x07request\x12G\n\x06\x63\x61ncel\x18\x03 \x01(\x0b\x32-.dokime.v1.ExecutionStopExecutionCancellationH\x00R\x06\x63\x61ncel\x12G\n\x06\x66inish\x18\x04 \x01(\x0b\x32-.dokime.v1.ExecutionStopExecutionCancellationH\x00R\x06\x66inishB\x07\n\x05value\"y\n\x1f\x45xecutionStopExecutionRunOutput\x12M\n\x08response\x18\x02 \x01(\x0b\x32/.dokime.v1.ExecutionMessages.StopExecution.Row2H\x00R\x08responseB\x07\n\x05value\"-\n+ExecutionSubmitExecutionCommandCancellation\"\xac\x02\n\'ExecutionSubmitExecutionCommandRunInput\x12T\n\x07request\x18\x01 \x01(\x0b\x32\x38.dokime.v1.ExecutionMessages.SubmitExecutionCommand.Row1H\x00R\x07request\x12P\n\x06\x63\x61ncel\x18\x03 \x01(\x0b\x32\x36.dokime.v1.ExecutionSubmitExecutionCommandCancellationH\x00R\x06\x63\x61ncel\x12P\n\x06\x66inish\x18\x04 \x01(\x0b\x32\x36.dokime.v1.ExecutionSubmitExecutionCommandCancellationH\x00R\x06\x66inishB\x07\n\x05value\"\x8b\x01\n(ExecutionSubmitExecutionCommandRunOutput\x12V\n\x08response\x18\x02 \x01(\x0b\x32\x38.dokime.v1.ExecutionMessages.SubmitExecutionCommand.Row2H\x00R\x08responseB\x07\n\x05value\"(\n&ExecutionUpdateCyclePolicyCancellation\"\x98\x02\n\"ExecutionUpdateCyclePolicyRunInput\x12O\n\x07request\x18\x01 \x01(\x0b\x32\x33.dokime.v1.ExecutionMessages.UpdateCyclePolicy.Row1H\x00R\x07request\x12K\n\x06\x63\x61ncel\x18\x03 \x01(\x0b\x32\x31.dokime.v1.ExecutionUpdateCyclePolicyCancellationH\x00R\x06\x63\x61ncel\x12K\n\x06\x66inish\x18\x04 \x01(\x0b\x32\x31.dokime.v1.ExecutionUpdateCyclePolicyCancellationH\x00R\x06\x66inishB\x07\n\x05value\"\x81\x01\n#ExecutionUpdateCyclePolicyRunOutput\x12Q\n\x08response\x18\x02 \x01(\x0b\x32\x33.dokime.v1.ExecutionMessages.UpdateCyclePolicy.Row2H\x00R\x08responseB\x07\n\x05value2\xd0\x32\n\tExecution\x12z\n\x10\x41ppendExecutions\x12\x32.dokime.v1.ExecutionMessages.AppendExecutions.Row1\x1a\x32.dokime.v1.ExecutionMessages.AppendExecutions.Row2\x12v\n\x13RunAppendExecutions\x12,.dokime.v1.ExecutionAppendExecutionsRunInput\x1a-.dokime.v1.ExecutionAppendExecutionsRunOutput(\x01\x30\x01\x12w\n\x0f\x43\x61ncelExecution\x12\x31.dokime.v1.ExecutionMessages.CancelExecution.Row1\x1a\x31.dokime.v1.ExecutionMessages.CancelExecution.Row2\x12s\n\x12RunCancelExecution\x12+.dokime.v1.ExecutionCancelExecutionRunInput\x1a,.dokime.v1.ExecutionCancelExecutionRunOutput(\x01\x30\x01\x12\x65\n\tCopyCycle\x12+.dokime.v1.ExecutionMessages.CopyCycle.Row1\x1a+.dokime.v1.ExecutionMessages.CopyCycle.Row2\x12\x61\n\x0cRunCopyCycle\x12%.dokime.v1.ExecutionCopyCycleRunInput\x1a&.dokime.v1.ExecutionCopyCycleRunOutput(\x01\x30\x01\x12k\n\x0b\x43reateCycle\x12-.dokime.v1.ExecutionMessages.CreateCycle.Row1\x1a-.dokime.v1.ExecutionMessages.CreateCycle.Row2\x12g\n\x0eRunCreateCycle\x12\'.dokime.v1.ExecutionCreateCycleRunInput\x1a(.dokime.v1.ExecutionCreateCycleRunOutput(\x01\x30\x01\x12h\n\nDeleteCase\x12,.dokime.v1.ExecutionMessages.DeleteCase.Row1\x1a,.dokime.v1.ExecutionMessages.DeleteCase.Row2\x12\x64\n\rRunDeleteCase\x12&.dokime.v1.ExecutionDeleteCaseRunInput\x1a\'.dokime.v1.ExecutionDeleteCaseRunOutput(\x01\x30\x01\x12_\n\x07GetCase\x12).dokime.v1.ExecutionMessages.GetCase.Row1\x1a).dokime.v1.ExecutionMessages.GetCase.Row2\x12[\n\nRunGetCase\x12#.dokime.v1.ExecutionGetCaseRunInput\x1a$.dokime.v1.ExecutionGetCaseRunOutput(\x01\x30\x01\x12\x62\n\x08GetCycle\x12*.dokime.v1.ExecutionMessages.GetCycle.Row1\x1a*.dokime.v1.ExecutionMessages.GetCycle.Row2\x12^\n\x0bRunGetCycle\x12$.dokime.v1.ExecutionGetCycleRunInput\x1a%.dokime.v1.ExecutionGetCycleRunOutput(\x01\x30\x01\x12n\n\x0cGetExecution\x12..dokime.v1.ExecutionMessages.GetExecution.Row1\x1a..dokime.v1.ExecutionMessages.GetExecution.Row2\x12j\n\x0fRunGetExecution\x12(.dokime.v1.ExecutionGetExecutionRunInput\x1a).dokime.v1.ExecutionGetExecutionRunOutput(\x01\x30\x01\x12\x83\x01\n\x13GetExecutionCommand\x12\x35.dokime.v1.ExecutionMessages.GetExecutionCommand.Row1\x1a\x35.dokime.v1.ExecutionMessages.GetExecutionCommand.Row2\x12\x7f\n\x16RunGetExecutionCommand\x12/.dokime.v1.ExecutionGetExecutionCommandRunInput\x1a\x30.dokime.v1.ExecutionGetExecutionCommandRunOutput(\x01\x30\x01\x12\x80\x01\n\x12ListCaseParameters\x12\x34.dokime.v1.ExecutionMessages.ListCaseParameters.Row1\x1a\x34.dokime.v1.ExecutionMessages.ListCaseParameters.Row2\x12|\n\x15RunListCaseParameters\x12..dokime.v1.ExecutionListCaseParametersRunInput\x1a/.dokime.v1.ExecutionListCaseParametersRunOutput(\x01\x30\x01\x12\x65\n\tListCases\x12+.dokime.v1.ExecutionMessages.ListCases.Row1\x1a+.dokime.v1.ExecutionMessages.ListCases.Row2\x12\x61\n\x0cRunListCases\x12%.dokime.v1.ExecutionListCasesRunInput\x1a&.dokime.v1.ExecutionListCasesRunOutput(\x01\x30\x01\x12h\n\nListCycles\x12,.dokime.v1.ExecutionMessages.ListCycles.Row1\x1a,.dokime.v1.ExecutionMessages.ListCycles.Row2\x12\x64\n\rRunListCycles\x12&.dokime.v1.ExecutionListCyclesRunInput\x1a\'.dokime.v1.ExecutionListCyclesRunOutput(\x01\x30\x01\x12\x8f\x01\n\x17ListExecutionParameters\x12\x39.dokime.v1.ExecutionMessages.ListExecutionParameters.Row1\x1a\x39.dokime.v1.ExecutionMessages.ListExecutionParameters.Row2\x12\x8b\x01\n\x1aRunListExecutionParameters\x12\x33.dokime.v1.ExecutionListExecutionParametersRunInput\x1a\x34.dokime.v1.ExecutionListExecutionParametersRunOutput(\x01\x30\x01\x12\x80\x01\n\x12ListExecutionSteps\x12\x34.dokime.v1.ExecutionMessages.ListExecutionSteps.Row1\x1a\x34.dokime.v1.ExecutionMessages.ListExecutionSteps.Row2\x12|\n\x15RunListExecutionSteps\x12..dokime.v1.ExecutionListExecutionStepsRunInput\x1a/.dokime.v1.ExecutionListExecutionStepsRunOutput(\x01\x30\x01\x12t\n\x0eListExecutions\x12\x30.dokime.v1.ExecutionMessages.ListExecutions.Row1\x1a\x30.dokime.v1.ExecutionMessages.ListExecutions.Row2\x12p\n\x11RunListExecutions\x12*.dokime.v1.ExecutionListExecutionsRunInput\x1a+.dokime.v1.ExecutionListExecutionsRunOutput(\x01\x30\x01\x12h\n\nPauseCycle\x12,.dokime.v1.ExecutionMessages.PauseCycle.Row1\x1a,.dokime.v1.ExecutionMessages.PauseCycle.Row2\x12\x64\n\rRunPauseCycle\x12&.dokime.v1.ExecutionPauseCycleRunInput\x1a\'.dokime.v1.ExecutionPauseCycleRunOutput(\x01\x30\x01\x12t\n\x0ePauseExecution\x12\x30.dokime.v1.ExecutionMessages.PauseExecution.Row1\x1a\x30.dokime.v1.ExecutionMessages.PauseExecution.Row2\x12p\n\x11RunPauseExecution\x12*.dokime.v1.ExecutionPauseExecutionRunInput\x1a+.dokime.v1.ExecutionPauseExecutionRunOutput(\x01\x30\x01\x12k\n\x0bRenameCycle\x12-.dokime.v1.ExecutionMessages.RenameCycle.Row1\x1a-.dokime.v1.ExecutionMessages.RenameCycle.Row2\x12g\n\x0eRunRenameCycle\x12\'.dokime.v1.ExecutionRenameCycleRunInput\x1a(.dokime.v1.ExecutionRenameCycleRunOutput(\x01\x30\x01\x12w\n\x0fRenameExecution\x12\x31.dokime.v1.ExecutionMessages.RenameExecution.Row1\x1a\x31.dokime.v1.ExecutionMessages.RenameExecution.Row2\x12s\n\x12RunRenameExecution\x12+.dokime.v1.ExecutionRenameExecutionRunInput\x1a,.dokime.v1.ExecutionRenameExecutionRunOutput(\x01\x30\x01\x12k\n\x0bResumeCycle\x12-.dokime.v1.ExecutionMessages.ResumeCycle.Row1\x1a-.dokime.v1.ExecutionMessages.ResumeCycle.Row2\x12g\n\x0eRunResumeCycle\x12\'.dokime.v1.ExecutionResumeCycleRunInput\x1a(.dokime.v1.ExecutionResumeCycleRunOutput(\x01\x30\x01\x12w\n\x0fResumeExecution\x12\x31.dokime.v1.ExecutionMessages.ResumeExecution.Row1\x1a\x31.dokime.v1.ExecutionMessages.ResumeExecution.Row2\x12s\n\x12RunResumeExecution\x12+.dokime.v1.ExecutionResumeExecutionRunInput\x1a,.dokime.v1.ExecutionResumeExecutionRunOutput(\x01\x30\x01\x12\x65\n\tSealCycle\x12+.dokime.v1.ExecutionMessages.SealCycle.Row1\x1a+.dokime.v1.ExecutionMessages.SealCycle.Row2\x12\x61\n\x0cRunSealCycle\x12%.dokime.v1.ExecutionSealCycleRunInput\x1a&.dokime.v1.ExecutionSealCycleRunOutput(\x01\x30\x01\x12z\n\x10SetCycleArchived\x12\x32.dokime.v1.ExecutionMessages.SetCycleArchived.Row1\x1a\x32.dokime.v1.ExecutionMessages.SetCycleArchived.Row2\x12v\n\x13RunSetCycleArchived\x12,.dokime.v1.ExecutionSetCycleArchivedRunInput\x1a-.dokime.v1.ExecutionSetCycleArchivedRunOutput(\x01\x30\x01\x12\x86\x01\n\x14SetExecutionArchived\x12\x36.dokime.v1.ExecutionMessages.SetExecutionArchived.Row1\x1a\x36.dokime.v1.ExecutionMessages.SetExecutionArchived.Row2\x12\x82\x01\n\x17RunSetExecutionArchived\x12\x30.dokime.v1.ExecutionSetExecutionArchivedRunInput\x1a\x31.dokime.v1.ExecutionSetExecutionArchivedRunOutput(\x01\x30\x01\x12t\n\x0eStartExecution\x12\x30.dokime.v1.ExecutionMessages.StartExecution.Row1\x1a\x30.dokime.v1.ExecutionMessages.StartExecution.Row2\x12p\n\x11RunStartExecution\x12*.dokime.v1.ExecutionStartExecutionRunInput\x1a+.dokime.v1.ExecutionStartExecutionRunOutput(\x01\x30\x01\x12q\n\rStopExecution\x12/.dokime.v1.ExecutionMessages.StopExecution.Row1\x1a/.dokime.v1.ExecutionMessages.StopExecution.Row2\x12m\n\x10RunStopExecution\x12).dokime.v1.ExecutionStopExecutionRunInput\x1a*.dokime.v1.ExecutionStopExecutionRunOutput(\x01\x30\x01\x12\x8c\x01\n\x16SubmitExecutionCommand\x12\x38.dokime.v1.ExecutionMessages.SubmitExecutionCommand.Row1\x1a\x38.dokime.v1.ExecutionMessages.SubmitExecutionCommand.Row2\x12\x88\x01\n\x19RunSubmitExecutionCommand\x12\x32.dokime.v1.ExecutionSubmitExecutionCommandRunInput\x1a\x33.dokime.v1.ExecutionSubmitExecutionCommandRunOutput(\x01\x30\x01\x12}\n\x11UpdateCyclePolicy\x12\x33.dokime.v1.ExecutionMessages.UpdateCyclePolicy.Row1\x1a\x33.dokime.v1.ExecutionMessages.UpdateCyclePolicy.Row2\x12y\n\x14RunUpdateCyclePolicy\x12-.dokime.v1.ExecutionUpdateCyclePolicyRunInput\x1a..dokime.v1.ExecutionUpdateCyclePolicyRunOutput(\x01\x30\x01\x62\x06proto3')

_globals = globals()
_builder.BuildMessageAndEnumDescriptors(DESCRIPTOR, _globals)
_pool.build(DESCRIPTOR, 'dokime_client._wire.dokime.v1.Execution_pb2', _globals)
if not _descriptor._USE_C_DESCRIPTORS:
  DESCRIPTOR._loaded_options = None
  _globals['_EXECUTIONAPPENDEXECUTIONSCANCELLATION']._serialized_start=2266
  _globals['_EXECUTIONAPPENDEXECUTIONSCANCELLATION']._serialized_end=2305
  _globals['_EXECUTIONAPPENDEXECUTIONSRUNINPUT']._serialized_start=2308
  _globals['_EXECUTIONAPPENDEXECUTIONSRUNINPUT']._serialized_end=2584
  _globals['_EXECUTIONAPPENDEXECUTIONSRUNOUTPUT']._serialized_start=2586
  _globals['_EXECUTIONAPPENDEXECUTIONSRUNOUTPUT']._serialized_end=2713
  _globals['_EXECUTIONCANCELEXECUTIONCANCELLATION']._serialized_start=2715
  _globals['_EXECUTIONCANCELEXECUTIONCANCELLATION']._serialized_end=2753
  _globals['_EXECUTIONCANCELEXECUTIONRUNINPUT']._serialized_start=2756
  _globals['_EXECUTIONCANCELEXECUTIONRUNINPUT']._serialized_end=3028
  _globals['_EXECUTIONCANCELEXECUTIONRUNOUTPUT']._serialized_start=3030
  _globals['_EXECUTIONCANCELEXECUTIONRUNOUTPUT']._serialized_end=3155
  _globals['_EXECUTIONCOPYCYCLECANCELLATION']._serialized_start=3157
  _globals['_EXECUTIONCOPYCYCLECANCELLATION']._serialized_end=3189
  _globals['_EXECUTIONCOPYCYCLERUNINPUT']._serialized_start=3192
  _globals['_EXECUTIONCOPYCYCLERUNINPUT']._serialized_end=3440
  _globals['_EXECUTIONCOPYCYCLERUNOUTPUT']._serialized_start=3442
  _globals['_EXECUTIONCOPYCYCLERUNOUTPUT']._serialized_end=3555
  _globals['_EXECUTIONCREATECYCLECANCELLATION']._serialized_start=3557
  _globals['_EXECUTIONCREATECYCLECANCELLATION']._serialized_end=3591
  _globals['_EXECUTIONCREATECYCLERUNINPUT']._serialized_start=3594
  _globals['_EXECUTIONCREATECYCLERUNINPUT']._serialized_end=3850
  _globals['_EXECUTIONCREATECYCLERUNOUTPUT']._serialized_start=3852
  _globals['_EXECUTIONCREATECYCLERUNOUTPUT']._serialized_end=3969
  _globals['_EXECUTIONDELETECASECANCELLATION']._serialized_start=3971
  _globals['_EXECUTIONDELETECASECANCELLATION']._serialized_end=4004
  _globals['_EXECUTIONDELETECASERUNINPUT']._serialized_start=4007
  _globals['_EXECUTIONDELETECASERUNINPUT']._serialized_end=4259
  _globals['_EXECUTIONDELETECASERUNOUTPUT']._serialized_start=4261
  _globals['_EXECUTIONDELETECASERUNOUTPUT']._serialized_end=4376
  _globals['_EXECUTIONGETCASECANCELLATION']._serialized_start=4378
  _globals['_EXECUTIONGETCASECANCELLATION']._serialized_end=4408
  _globals['_EXECUTIONGETCASERUNINPUT']._serialized_start=4411
  _globals['_EXECUTIONGETCASERUNINPUT']._serialized_end=4651
  _globals['_EXECUTIONGETCASERUNOUTPUT']._serialized_start=4653
  _globals['_EXECUTIONGETCASERUNOUTPUT']._serialized_end=4762
  _globals['_EXECUTIONGETCYCLECANCELLATION']._serialized_start=4764
  _globals['_EXECUTIONGETCYCLECANCELLATION']._serialized_end=4795
  _globals['_EXECUTIONGETCYCLERUNINPUT']._serialized_start=4798
  _globals['_EXECUTIONGETCYCLERUNINPUT']._serialized_end=5042
  _globals['_EXECUTIONGETCYCLERUNOUTPUT']._serialized_start=5044
  _globals['_EXECUTIONGETCYCLERUNOUTPUT']._serialized_end=5155
  _globals['_EXECUTIONGETEXECUTIONCANCELLATION']._serialized_start=5157
  _globals['_EXECUTIONGETEXECUTIONCANCELLATION']._serialized_end=5192
  _globals['_EXECUTIONGETEXECUTIONRUNINPUT']._serialized_start=5195
  _globals['_EXECUTIONGETEXECUTIONRUNINPUT']._serialized_end=5455
  _globals['_EXECUTIONGETEXECUTIONRUNOUTPUT']._serialized_start=5457
  _globals['_EXECUTIONGETEXECUTIONRUNOUTPUT']._serialized_end=5576
  _globals['_EXECUTIONGETEXECUTIONCOMMANDCANCELLATION']._serialized_start=5578
  _globals['_EXECUTIONGETEXECUTIONCOMMANDCANCELLATION']._serialized_end=5620
  _globals['_EXECUTIONGETEXECUTIONCOMMANDRUNINPUT']._serialized_start=5623
  _globals['_EXECUTIONGETEXECUTIONCOMMANDRUNINPUT']._serialized_end=5911
  _globals['_EXECUTIONGETEXECUTIONCOMMANDRUNOUTPUT']._serialized_start=5914
  _globals['_EXECUTIONGETEXECUTIONCOMMANDRUNOUTPUT']._serialized_end=6047
  _globals['_EXECUTIONLISTCASEPARAMETERSCANCELLATION']._serialized_start=6049
  _globals['_EXECUTIONLISTCASEPARAMETERSCANCELLATION']._serialized_end=6090
  _globals['_EXECUTIONLISTCASEPARAMETERSRUNINPUT']._serialized_start=6093
  _globals['_EXECUTIONLISTCASEPARAMETERSRUNINPUT']._serialized_end=6377
  _globals['_EXECUTIONLISTCASEPARAMETERSRUNOUTPUT']._serialized_start=6380
  _globals['_EXECUTIONLISTCASEPARAMETERSRUNOUTPUT']._serialized_end=6511
  _globals['_EXECUTIONLISTCASESCANCELLATION']._serialized_start=6513
  _globals['_EXECUTIONLISTCASESCANCELLATION']._serialized_end=6545
  _globals['_EXECUTIONLISTCASESRUNINPUT']._serialized_start=6548
  _globals['_EXECUTIONLISTCASESRUNINPUT']._serialized_end=6796
  _globals['_EXECUTIONLISTCASESRUNOUTPUT']._serialized_start=6798
  _globals['_EXECUTIONLISTCASESRUNOUTPUT']._serialized_end=6911
  _globals['_EXECUTIONLISTCYCLESCANCELLATION']._serialized_start=6913
  _globals['_EXECUTIONLISTCYCLESCANCELLATION']._serialized_end=6946
  _globals['_EXECUTIONLISTCYCLESRUNINPUT']._serialized_start=6949
  _globals['_EXECUTIONLISTCYCLESRUNINPUT']._serialized_end=7201
  _globals['_EXECUTIONLISTCYCLESRUNOUTPUT']._serialized_start=7203
  _globals['_EXECUTIONLISTCYCLESRUNOUTPUT']._serialized_end=7318
  _globals['_EXECUTIONLISTEXECUTIONPARAMETERSCANCELLATION']._serialized_start=7320
  _globals['_EXECUTIONLISTEXECUTIONPARAMETERSCANCELLATION']._serialized_end=7366
  _globals['_EXECUTIONLISTEXECUTIONPARAMETERSRUNINPUT']._serialized_start=7369
  _globals['_EXECUTIONLISTEXECUTIONPARAMETERSRUNINPUT']._serialized_end=7673
  _globals['_EXECUTIONLISTEXECUTIONPARAMETERSRUNOUTPUT']._serialized_start=7676
  _globals['_EXECUTIONLISTEXECUTIONPARAMETERSRUNOUTPUT']._serialized_end=7817
  _globals['_EXECUTIONLISTEXECUTIONSTEPSCANCELLATION']._serialized_start=7819
  _globals['_EXECUTIONLISTEXECUTIONSTEPSCANCELLATION']._serialized_end=7860
  _globals['_EXECUTIONLISTEXECUTIONSTEPSRUNINPUT']._serialized_start=7863
  _globals['_EXECUTIONLISTEXECUTIONSTEPSRUNINPUT']._serialized_end=8147
  _globals['_EXECUTIONLISTEXECUTIONSTEPSRUNOUTPUT']._serialized_start=8150
  _globals['_EXECUTIONLISTEXECUTIONSTEPSRUNOUTPUT']._serialized_end=8281
  _globals['_EXECUTIONLISTEXECUTIONSCANCELLATION']._serialized_start=8283
  _globals['_EXECUTIONLISTEXECUTIONSCANCELLATION']._serialized_end=8320
  _globals['_EXECUTIONLISTEXECUTIONSRUNINPUT']._serialized_start=8323
  _globals['_EXECUTIONLISTEXECUTIONSRUNINPUT']._serialized_end=8591
  _globals['_EXECUTIONLISTEXECUTIONSRUNOUTPUT']._serialized_start=8593
  _globals['_EXECUTIONLISTEXECUTIONSRUNOUTPUT']._serialized_end=8716
  _globals['_EXECUTIONPAUSECYCLECANCELLATION']._serialized_start=8718
  _globals['_EXECUTIONPAUSECYCLECANCELLATION']._serialized_end=8751
  _globals['_EXECUTIONPAUSECYCLERUNINPUT']._serialized_start=8754
  _globals['_EXECUTIONPAUSECYCLERUNINPUT']._serialized_end=9006
  _globals['_EXECUTIONPAUSECYCLERUNOUTPUT']._serialized_start=9008
  _globals['_EXECUTIONPAUSECYCLERUNOUTPUT']._serialized_end=9123
  _globals['_EXECUTIONPAUSEEXECUTIONCANCELLATION']._serialized_start=9125
  _globals['_EXECUTIONPAUSEEXECUTIONCANCELLATION']._serialized_end=9162
  _globals['_EXECUTIONPAUSEEXECUTIONRUNINPUT']._serialized_start=9165
  _globals['_EXECUTIONPAUSEEXECUTIONRUNINPUT']._serialized_end=9433
  _globals['_EXECUTIONPAUSEEXECUTIONRUNOUTPUT']._serialized_start=9435
  _globals['_EXECUTIONPAUSEEXECUTIONRUNOUTPUT']._serialized_end=9558
  _globals['_EXECUTIONRENAMECYCLECANCELLATION']._serialized_start=9560
  _globals['_EXECUTIONRENAMECYCLECANCELLATION']._serialized_end=9594
  _globals['_EXECUTIONRENAMECYCLERUNINPUT']._serialized_start=9597
  _globals['_EXECUTIONRENAMECYCLERUNINPUT']._serialized_end=9853
  _globals['_EXECUTIONRENAMECYCLERUNOUTPUT']._serialized_start=9855
  _globals['_EXECUTIONRENAMECYCLERUNOUTPUT']._serialized_end=9972
  _globals['_EXECUTIONRENAMEEXECUTIONCANCELLATION']._serialized_start=9974
  _globals['_EXECUTIONRENAMEEXECUTIONCANCELLATION']._serialized_end=10012
  _globals['_EXECUTIONRENAMEEXECUTIONRUNINPUT']._serialized_start=10015
  _globals['_EXECUTIONRENAMEEXECUTIONRUNINPUT']._serialized_end=10287
  _globals['_EXECUTIONRENAMEEXECUTIONRUNOUTPUT']._serialized_start=10289
  _globals['_EXECUTIONRENAMEEXECUTIONRUNOUTPUT']._serialized_end=10414
  _globals['_EXECUTIONRESUMECYCLECANCELLATION']._serialized_start=10416
  _globals['_EXECUTIONRESUMECYCLECANCELLATION']._serialized_end=10450
  _globals['_EXECUTIONRESUMECYCLERUNINPUT']._serialized_start=10453
  _globals['_EXECUTIONRESUMECYCLERUNINPUT']._serialized_end=10709
  _globals['_EXECUTIONRESUMECYCLERUNOUTPUT']._serialized_start=10711
  _globals['_EXECUTIONRESUMECYCLERUNOUTPUT']._serialized_end=10828
  _globals['_EXECUTIONRESUMEEXECUTIONCANCELLATION']._serialized_start=10830
  _globals['_EXECUTIONRESUMEEXECUTIONCANCELLATION']._serialized_end=10868
  _globals['_EXECUTIONRESUMEEXECUTIONRUNINPUT']._serialized_start=10871
  _globals['_EXECUTIONRESUMEEXECUTIONRUNINPUT']._serialized_end=11143
  _globals['_EXECUTIONRESUMEEXECUTIONRUNOUTPUT']._serialized_start=11145
  _globals['_EXECUTIONRESUMEEXECUTIONRUNOUTPUT']._serialized_end=11270
  _globals['_EXECUTIONSEALCYCLECANCELLATION']._serialized_start=11272
  _globals['_EXECUTIONSEALCYCLECANCELLATION']._serialized_end=11304
  _globals['_EXECUTIONSEALCYCLERUNINPUT']._serialized_start=11307
  _globals['_EXECUTIONSEALCYCLERUNINPUT']._serialized_end=11555
  _globals['_EXECUTIONSEALCYCLERUNOUTPUT']._serialized_start=11557
  _globals['_EXECUTIONSEALCYCLERUNOUTPUT']._serialized_end=11670
  _globals['_EXECUTIONSETCYCLEARCHIVEDCANCELLATION']._serialized_start=11672
  _globals['_EXECUTIONSETCYCLEARCHIVEDCANCELLATION']._serialized_end=11711
  _globals['_EXECUTIONSETCYCLEARCHIVEDRUNINPUT']._serialized_start=11714
  _globals['_EXECUTIONSETCYCLEARCHIVEDRUNINPUT']._serialized_end=11990
  _globals['_EXECUTIONSETCYCLEARCHIVEDRUNOUTPUT']._serialized_start=11992
  _globals['_EXECUTIONSETCYCLEARCHIVEDRUNOUTPUT']._serialized_end=12119
  _globals['_EXECUTIONSETEXECUTIONARCHIVEDCANCELLATION']._serialized_start=12121
  _globals['_EXECUTIONSETEXECUTIONARCHIVEDCANCELLATION']._serialized_end=12164
  _globals['_EXECUTIONSETEXECUTIONARCHIVEDRUNINPUT']._serialized_start=12167
  _globals['_EXECUTIONSETEXECUTIONARCHIVEDRUNINPUT']._serialized_end=12459
  _globals['_EXECUTIONSETEXECUTIONARCHIVEDRUNOUTPUT']._serialized_start=12462
  _globals['_EXECUTIONSETEXECUTIONARCHIVEDRUNOUTPUT']._serialized_end=12597
  _globals['_EXECUTIONSTARTEXECUTIONCANCELLATION']._serialized_start=12599
  _globals['_EXECUTIONSTARTEXECUTIONCANCELLATION']._serialized_end=12636
  _globals['_EXECUTIONSTARTEXECUTIONRUNINPUT']._serialized_start=12639
  _globals['_EXECUTIONSTARTEXECUTIONRUNINPUT']._serialized_end=12907
  _globals['_EXECUTIONSTARTEXECUTIONRUNOUTPUT']._serialized_start=12909
  _globals['_EXECUTIONSTARTEXECUTIONRUNOUTPUT']._serialized_end=13032
  _globals['_EXECUTIONSTOPEXECUTIONCANCELLATION']._serialized_start=13034
  _globals['_EXECUTIONSTOPEXECUTIONCANCELLATION']._serialized_end=13070
  _globals['_EXECUTIONSTOPEXECUTIONRUNINPUT']._serialized_start=13073
  _globals['_EXECUTIONSTOPEXECUTIONRUNINPUT']._serialized_end=13337
  _globals['_EXECUTIONSTOPEXECUTIONRUNOUTPUT']._serialized_start=13339
  _globals['_EXECUTIONSTOPEXECUTIONRUNOUTPUT']._serialized_end=13460
  _globals['_EXECUTIONSUBMITEXECUTIONCOMMANDCANCELLATION']._serialized_start=13462
  _globals['_EXECUTIONSUBMITEXECUTIONCOMMANDCANCELLATION']._serialized_end=13507
  _globals['_EXECUTIONSUBMITEXECUTIONCOMMANDRUNINPUT']._serialized_start=13510
  _globals['_EXECUTIONSUBMITEXECUTIONCOMMANDRUNINPUT']._serialized_end=13810
  _globals['_EXECUTIONSUBMITEXECUTIONCOMMANDRUNOUTPUT']._serialized_start=13813
  _globals['_EXECUTIONSUBMITEXECUTIONCOMMANDRUNOUTPUT']._serialized_end=13952
  _globals['_EXECUTIONUPDATECYCLEPOLICYCANCELLATION']._serialized_start=13954
  _globals['_EXECUTIONUPDATECYCLEPOLICYCANCELLATION']._serialized_end=13994
  _globals['_EXECUTIONUPDATECYCLEPOLICYRUNINPUT']._serialized_start=13997
  _globals['_EXECUTIONUPDATECYCLEPOLICYRUNINPUT']._serialized_end=14277
  _globals['_EXECUTIONUPDATECYCLEPOLICYRUNOUTPUT']._serialized_start=14280
  _globals['_EXECUTIONUPDATECYCLEPOLICYRUNOUTPUT']._serialized_end=14409
  _globals['_EXECUTION']._serialized_start=14412
  _globals['_EXECUTION']._serialized_end=20892
# @@protoc_insertion_point(module_scope)

# === Types === #

# === Classes === #

# === Public Functions === #

# === Private Functions === #

# === Entry Point === #
