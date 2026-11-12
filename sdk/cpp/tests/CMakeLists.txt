add_executable(dokime-builders builders.cpp)
target_link_libraries(dokime-builders PRIVATE Dokime::Cpp)
add_test(NAME builders COMMAND dokime-builders)

# A Test-Owned Service Proves Generated Stub Behaviour Until Product Services Land.
add_library(dokime_test_protocol STATIC)
protobuf_generate(TARGET dokime_test_protocol LANGUAGE cpp
  PROTOS "${CMAKE_CURRENT_SOURCE_DIR}/proto/journey.proto"
  IMPORT_DIRS "${CMAKE_CURRENT_SOURCE_DIR}/proto" "${DOKIME_PROTO_ROOT}" "${DOKIME_PROTOBUF_INCLUDE}"
  PROTOC_OUT_DIR "${DOKIME_GENERATED}" PROTOC_EXE "$<TARGET_FILE:protoc>")
protobuf_generate(TARGET dokime_test_protocol LANGUAGE grpc
  PROTOS "${CMAKE_CURRENT_SOURCE_DIR}/proto/journey.proto"
  IMPORT_DIRS "${CMAKE_CURRENT_SOURCE_DIR}/proto" "${DOKIME_PROTO_ROOT}" "${DOKIME_PROTOBUF_INCLUDE}"
  PROTOC_OUT_DIR "${DOKIME_GENERATED}" PROTOC_EXE "$<TARGET_FILE:protoc>"
  GENERATE_EXTENSIONS .grpc.pb.h .grpc.pb.cc PLUGIN "protoc-gen-grpc=$<TARGET_FILE:grpc_cpp_plugin>")
target_link_libraries(dokime_test_protocol PUBLIC Dokime::Cpp)
add_executable(dokime-client-journey integration/client_reads_backend.cpp)
target_link_libraries(dokime-client-journey PRIVATE dokime_test_protocol)
add_test(NAME client_reads_backend COMMAND dokime-client-journey)
add_executable(dokime-authoring-journey integration/author_materializes_suite.cpp)
target_link_libraries(dokime-authoring-journey PRIVATE Dokime::Cpp)
add_test(NAME author_materializes_suite COMMAND dokime-authoring-journey)
set_tests_properties(builders client_reads_backend author_materializes_suite PROPERTIES TIMEOUT 240)

find_package(Python3 COMPONENTS Interpreter REQUIRED)
add_executable(dokime-story-plugin ../starter/main.cpp)
target_link_libraries(dokime-story-plugin PRIVATE Dokime::Cpp)
add_executable(dokime-plugin-journey integration/plugin_executes_case.cpp)
target_link_libraries(dokime-plugin-journey PRIVATE Dokime::Cpp)
foreach(placement managed remote)
  add_test(NAME plugin_executes_case_${placement}
    COMMAND "${Python3_EXECUTABLE}" "${CMAKE_CURRENT_SOURCE_DIR}/support/run_plugin.py"
      $<TARGET_FILE:dokime-story-plugin> $<TARGET_FILE:dokime-plugin-journey> ${placement})
  set_tests_properties(plugin_executes_case_${placement} PROPERTIES TIMEOUT 240)
endforeach()

if(DOKIME_BUILD_CONFORMANCE)
  # Reuse the Reference Fixture Source, Replacing Only Its Plugin Composition.
  file(READ "${harmos_SOURCE_DIR}/sdk/cpp/examples/conformance/main.cpp" fixture)
  string(REPLACE "harmos::Plugin plugin" "dokime::Plugin plugin" fixture "${fixture}")
  string(REPLACE "#include <iostream>" "#include <iostream>\n#include <dokime/plugin.hpp>" fixture "${fixture}")
  file(CONFIGURE OUTPUT "${CMAKE_CURRENT_BINARY_DIR}/conformance.cpp" CONTENT "${fixture}" @ONLY)
  add_library(dokime_conformance_protocol STATIC)
  protobuf_generate(TARGET dokime_conformance_protocol LANGUAGE cpp
    PROTOS "${harmos_SOURCE_DIR}/sdk/cpp/proto/harmos/conformance/v1/fixture.proto"
    IMPORT_DIRS "${harmos_SOURCE_DIR}/sdk/cpp/proto" "${DOKIME_PROTOBUF_INCLUDE}"
    PROTOC_OUT_DIR "${DOKIME_GENERATED}" PROTOC_EXE "$<TARGET_FILE:protoc>")
  target_link_libraries(dokime_conformance_protocol PUBLIC Dokime::Cpp)
  add_executable(dokime-conformance "${CMAKE_CURRENT_BINARY_DIR}/conformance.cpp")
  target_link_libraries(dokime-conformance PRIVATE dokime_conformance_protocol)
endif()
add_executable(dokime-system-suite system/main.cpp)
target_link_libraries(dokime-system-suite PRIVATE Dokime::Cpp)
