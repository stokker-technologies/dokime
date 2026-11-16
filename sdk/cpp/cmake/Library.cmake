set(DOKIME_PROTO_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../../proto" CACHE PATH "Authoritative Dokime Protocol Sources")
set(DOKIME_GENERATED "${CMAKE_CURRENT_BINARY_DIR}/g")
# Explicit Protocol Inputs Keep Ninja Freshness Checks Free of Forced Regeneration.
set(protocols
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/aggregates.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/backend.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/backend/overview.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/cases.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/content/artifacts.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/content/execution_files.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/cycles.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/documents.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/effects.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/evidence.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/evidence/health_events.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/evidence/logs.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/evidence/notes.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/evidence/records.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/evidence/samples.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/evidence/traceability.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/executions.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/executions/commands.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/executions/progress.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/executions/schemas.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/files.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/files/captures.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/files/uploads.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/media/captures.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/plots/settings.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/plots/snapshots.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/plugins/secrets.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/prompt_definitions.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/prompts.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/prompts/answers.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/record_schemas.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/reports.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/secrets.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/settings.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/storage/sources.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/stream_schemas.proto"
  "${DOKIME_PROTO_ROOT}/dokime/host/v1/suites.proto"
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
add_library(dokime_protocol STATIC)
dokime_protocol(dokime_protocol "${DOKIME_PROTO_ROOT}" "${DOKIME_GENERATED}" ${protocols})
target_link_libraries(dokime_protocol PUBLIC Harmos::Cpp)
add_library(dokime_cpp STATIC src/plugin.cpp src/context.cpp src/client.cpp src/generated/host.cpp)
add_library(Dokime::Cpp ALIAS dokime_cpp)
set_target_properties(dokime_cpp PROPERTIES EXPORT_NAME Cpp)
target_compile_features(dokime_cpp PUBLIC cxx_std_20)
target_include_directories(dokime_cpp PUBLIC
  $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
  $<INSTALL_INTERFACE:${CMAKE_INSTALL_INCLUDEDIR}>)
target_link_libraries(dokime_cpp PUBLIC dokime_protocol PRIVATE OpenSSL::Crypto)
if(MSVC)
  target_compile_options(dokime_cpp PRIVATE /W4 /WX /EHsc /Zc:preprocessor)
else()
  target_compile_options(dokime_cpp PRIVATE -Wall -Wextra -Werror)
endif()
