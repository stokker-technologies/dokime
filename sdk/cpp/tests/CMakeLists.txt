# Compile the Generated Public Barrels Without Linking Handwritten SDK Consumers.
add_library(dokime-generation OBJECT generation/surface.cpp)
target_link_libraries(dokime-generation PRIVATE Dokime::Cpp)

find_package(Catch2 3 CONFIG REQUIRED)
include(Catch)
add_executable(dokime-builders builders.cpp)
target_link_libraries(dokime-builders PRIVATE Dokime::Cpp Catch2::Catch2WithMain)
add_test(NAME builders COMMAND dokime-builders)

add_executable(dokime-client-journey integration/client_reads_backend.cpp)
target_link_libraries(dokime-client-journey PRIVATE Dokime::Cpp Catch2::Catch2WithMain)
add_test(NAME client_reads_backend COMMAND dokime-client-journey)
add_executable(dokime-authoring-journey authoring.cpp)
target_link_libraries(dokime-authoring-journey PRIVATE Dokime::Cpp Catch2::Catch2WithMain)
add_test(NAME author_materializes_suite COMMAND dokime-authoring-journey)
set_tests_properties(builders client_reads_backend author_materializes_suite PROPERTIES TIMEOUT 240)

find_package(Python3 COMPONENTS Interpreter REQUIRED)
add_executable(dokime-story-plugin ../examples/test/main.cpp)
target_link_libraries(dokime-story-plugin PRIVATE Dokime::Cpp)
add_executable(dokime-plugin-journey integration/capture_roundtrip.cpp)
target_link_libraries(dokime-plugin-journey PRIVATE Dokime::Cpp Catch2::Catch2)
foreach(placement managed remote)
  add_test(NAME plugin_executes_case_${placement}
    COMMAND "${Python3_EXECUTABLE}" "${CMAKE_CURRENT_SOURCE_DIR}/support/run_plugin.py"
      $<TARGET_FILE:dokime-story-plugin> $<TARGET_FILE:dokime-plugin-journey> ${placement})
  set_tests_properties(plugin_executes_case_${placement} PROPERTIES TIMEOUT 240)
endforeach()

option(DOKIME_BUILD_CONFORMANCE "Build the Library Conformance Plugin" ON)
if(DOKIME_BUILD_CONFORMANCE)
  add_subdirectory("${harmos_SOURCE_DIR}/sdk/cpp/examples/conformance" "${CMAKE_CURRENT_BINARY_DIR}/conformance")
  set_target_properties(harmos-cpp-conformance PROPERTIES
    OUTPUT_NAME dokime-conformance RUNTIME_OUTPUT_DIRECTORY "${CMAKE_CURRENT_BINARY_DIR}")
endif()
set(thermal "${CMAKE_CURRENT_SOURCE_DIR}/../../tests/fixtures/cpp/thermal")
add_library(dokime_thermal_protocol STATIC)
set(thermal_generated "${CMAKE_CURRENT_BINARY_DIR}/g")
file(MAKE_DIRECTORY "${thermal_generated}")
protobuf_generate(TARGET dokime_thermal_protocol LANGUAGE cpp
  PROTOS "${thermal}/thermal.proto" IMPORT_DIRS "${thermal}"
  PROTOC_OUT_DIR "${thermal_generated}")
target_include_directories(dokime_thermal_protocol SYSTEM PUBLIC "${thermal_generated}")
target_link_libraries(dokime_thermal_protocol PUBLIC Dokime::Cpp)
add_executable(dokime-system-suite "${thermal}/main.cpp")
target_link_libraries(dokime-system-suite PRIVATE dokime_thermal_protocol)

add_executable(dokime-value-plugin support/value_plugin.cpp)
target_link_libraries(dokime-value-plugin PRIVATE Dokime::Cpp)
add_executable(dokime-values-journey integration/values_roundtrip.cpp)
target_link_libraries(dokime-values-journey PRIVATE Dokime::Cpp Catch2::Catch2)
add_test(NAME values_roundtrip
  COMMAND "${Python3_EXECUTABLE}" "${CMAKE_CURRENT_SOURCE_DIR}/support/run_plugin.py"
    $<TARGET_FILE:dokime-value-plugin> $<TARGET_FILE:dokime-values-journey> managed)
set_tests_properties(values_roundtrip PROPERTIES TIMEOUT 240)
