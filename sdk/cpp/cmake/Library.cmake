set(DOKIME_PROTO_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../../proto" CACHE PATH "Authoritative Dokime Protocol Sources")
set(DOKIME_GENERATED "${CMAKE_CURRENT_BINARY_DIR}/g")
# Explicit Protocol Inputs Keep Ninja Freshness Checks Free of Forced Regeneration.
# Plugins Compile Only the Message Closure of Their Plugin Contract and Host Operations.
set(plugin_protocols
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/backend.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/content/artifacts.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/cycles.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/documents.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/evidence.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/evidence/logs.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/evidence/records.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/evidence/traceability.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/executions.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/executions/progress.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/files.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/files/captures.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/files/uploads.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/plots/snapshots.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/prompts.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/record_schemas.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/secrets.proto"
  "${DOKIME_PROTO_ROOT}/dokime/model/v1/documents.proto"
  "${DOKIME_PROTO_ROOT}/dokime/model/v1/evidence.proto"
  "${DOKIME_PROTO_ROOT}/dokime/model/v1/execution.proto"
  "${DOKIME_PROTO_ROOT}/dokime/model/v1/expressions.proto"
  "${DOKIME_PROTO_ROOT}/dokime/model/v1/files.proto"
  "${DOKIME_PROTO_ROOT}/dokime/model/v1/metadata.proto"
  "${DOKIME_PROTO_ROOT}/dokime/model/v1/parameters.proto"
  "${DOKIME_PROTO_ROOT}/dokime/model/v1/plots.proto"
  "${DOKIME_PROTO_ROOT}/dokime/model/v1/prompts.proto"
  "${DOKIME_PROTO_ROOT}/dokime/model/v1/record_contracts.proto"
  "${DOKIME_PROTO_ROOT}/dokime/model/v1/scalar.proto"
  "${DOKIME_PROTO_ROOT}/dokime/model/v1/secrets.proto"
  "${DOKIME_PROTO_ROOT}/dokime/model/v1/suites.proto"
  "${DOKIME_PROTO_ROOT}/dokime/model/v1/traceability.proto"
  "${DOKIME_PROTO_ROOT}/dokime/plugin/v1/criteria.proto"
  "${DOKIME_PROTO_ROOT}/dokime/plugin/v1/executions.proto"
  "${DOKIME_PROTO_ROOT}/dokime/plugin/v1/steps.proto"
  "${DOKIME_PROTO_ROOT}/dokime/plugin/v1/test_cases.proto")
# Client-Only Messages; the Shared Generated Barrel Still Declares Them for Plugin Headers.
set(client_protocols
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/access/certificates.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/access/listener.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/access/pairings.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/aggregates.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/backend/overview.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/cases.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/content/execution_files.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/evidence/health_events.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/evidence/notes.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/evidence/samples.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/executions/commands.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/executions/schemas.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/invocations.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/media/captures.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/operations.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/plots/settings.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/plugins.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/plugins/secrets.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/plugins/settings.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/prompt_definitions.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/prompts/answers.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/reports.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/settings.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/status.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/status/diagnostics.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/status/logs.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/status/trace_events.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/storage/sources.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/stream_schemas.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/suites.proto")
# Services the Client Calls; Plugins Reach the Host Through the Library Carrier.
set(service_protocols
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/access/certificates.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/access/listener.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/access/pairings.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/aggregates.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/backend.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/backend/overview.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/cases.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/content/artifacts.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/content/execution_files.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/cycles.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/documents.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/evidence.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/evidence/health_events.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/evidence/notes.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/evidence/records.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/evidence/samples.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/evidence/traceability.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/executions.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/executions/commands.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/executions/progress.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/executions/schemas.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/files.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/files/uploads.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/invocations.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/media/captures.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/operations.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/plots/settings.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/plots/snapshots.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/plugins.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/plugins/secrets.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/plugins/settings.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/prompt_definitions.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/prompts.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/prompts/answers.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/record_schemas.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/reports.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/secrets.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/settings.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/status.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/status/diagnostics.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/status/logs.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/status/trace_events.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/storage/sources.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/stream_schemas.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/suites.proto")
dokime_generate(plugin_messages cpp ${plugin_protocols})
dokime_generate(client_messages cpp ${client_protocols})
dokime_generate(client_services grpc ${service_protocols})
set(client_headers ${client_messages})
list(FILTER client_headers INCLUDE REGEX "\\.h$")
dokime_protocol(dokime_plugin_protocol ${plugin_messages} ${client_headers})
target_link_libraries(dokime_plugin_protocol PUBLIC Harmos::Cpp protobuf::libprotobuf)
dokime_protocol(dokime_client_protocol ${client_messages} ${client_services})
target_link_libraries(dokime_client_protocol PUBLIC dokime_plugin_protocol gRPC::grpc++)
target_precompile_headers(dokime_client_protocol PRIVATE <grpcpp/server_context.h> <grpcpp/support/async_stream.h>
  <grpcpp/support/async_unary_call.h> <grpcpp/support/client_callback.h> <grpcpp/support/server_callback.h>
  <grpcpp/impl/server_callback_handlers.h> <grpcpp/support/sync_stream.h>)

# Keep Authored SDK Diagnostics Identical Across Both Libraries.
if(MSVC)
  set(DOKIME_CXX_WARNINGS /W4 /WX /EHsc /Zc:preprocessor)
else()
  set(DOKIME_CXX_WARNINGS -Wall -Wextra -Werror)
endif()
add_library(dokime_plugin STATIC src/plugin.cpp src/context.cpp src/operations.cpp src/configuration.cpp src/errors.cpp
  src/transfers.cpp src/generated/host.cpp)
add_library(Dokime::Plugin ALIAS dokime_plugin)
set_target_properties(dokime_plugin PROPERTIES EXPORT_NAME Plugin)
target_compile_features(dokime_plugin PUBLIC cxx_std_20)
target_include_directories(dokime_plugin PUBLIC
  $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
  $<INSTALL_INTERFACE:${CMAKE_INSTALL_INCLUDEDIR}>)
target_link_libraries(dokime_plugin PUBLIC dokime_plugin_protocol)
target_compile_options(dokime_plugin PRIVATE ${DOKIME_CXX_WARNINGS})
add_library(dokime_client STATIC src/client.cpp src/handles.cpp src/connection.cpp)
add_library(Dokime::Client ALIAS dokime_client)
set_target_properties(dokime_client PROPERTIES EXPORT_NAME Client)
target_link_libraries(dokime_client PUBLIC dokime_plugin dokime_client_protocol PRIVATE OpenSSL::Crypto)
target_compile_options(dokime_client PRIVATE ${DOKIME_CXX_WARNINGS})

target_compile_definitions(dokime_client PRIVATE DOKIME_SDK_VERSION="${DokimeCpp_VERSION}")
