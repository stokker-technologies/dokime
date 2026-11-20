# Compile the Generated Public Barrels Without Linking Handwritten SDK Consumers.
add_library(dokime-generation OBJECT generation/surface.cpp)
target_link_libraries(dokime-generation PRIVATE Dokime::Client)

find_package(Catch2 3 CONFIG REQUIRED)
include(Catch)
add_executable(dokime-builders builders.cpp configuration.cpp connection.cpp handles.cpp transfers.cpp)
target_link_libraries(dokime-builders PRIVATE Dokime::Client Catch2::Catch2WithMain)
add_test(NAME builders COMMAND dokime-builders)

add_executable(dokime-client-journey integration/client_reads_backend.cpp)
target_link_libraries(dokime-client-journey PRIVATE Dokime::Client Catch2::Catch2WithMain)
add_test(NAME client_reads_backend COMMAND dokime-client-journey)
add_executable(dokime-authoring-journey authoring.cpp)
target_link_libraries(dokime-authoring-journey PRIVATE Dokime::Plugin Catch2::Catch2WithMain)
add_test(NAME author_materializes_suite COMMAND dokime-authoring-journey)
set_tests_properties(builders client_reads_backend author_materializes_suite PROPERTIES TIMEOUT 240)

find_package(Python3 COMPONENTS Interpreter REQUIRED)
add_executable(dokime-story-plugin ../examples/starter/src/plugin/main.cpp)
target_link_libraries(dokime-story-plugin PRIVATE starter-project Dokime::Client)
add_executable(dokime-plugin-journey integration/capture_roundtrip.cpp)
target_link_libraries(dokime-plugin-journey PRIVATE Dokime::Plugin Catch2::Catch2)
foreach(placement managed remote)
  add_test(NAME plugin_executes_case_${placement}
    COMMAND "${Python3_EXECUTABLE}" "${CMAKE_CURRENT_SOURCE_DIR}/support/run_plugin.py"
      $<TARGET_FILE:dokime-story-plugin> $<TARGET_FILE:dokime-plugin-journey> ${placement})
  set_tests_properties(plugin_executes_case_${placement} PROPERTIES TIMEOUT 240)
endforeach()

# The Thermal Fixture Is a Standalone Project That Also Builds Against the Bundled Package.
add_subdirectory("${CMAKE_CURRENT_SOURCE_DIR}/../../tests/fixtures/cpp/thermal" thermal)

add_executable(dokime-value-plugin support/value_plugin.cpp)
target_link_libraries(dokime-value-plugin PRIVATE Dokime::Client)
add_executable(dokime-values-journey integration/values_roundtrip.cpp)
target_link_libraries(dokime-values-journey PRIVATE Dokime::Plugin Catch2::Catch2)
add_test(NAME values_roundtrip
  COMMAND "${Python3_EXECUTABLE}" "${CMAKE_CURRENT_SOURCE_DIR}/support/run_plugin.py"
    $<TARGET_FILE:dokime-value-plugin> $<TARGET_FILE:dokime-values-journey> managed)
set_tests_properties(values_roundtrip PROPERTIES TIMEOUT 240)

add_executable(dokime-lifecycle-plugin support/lifecycle_plugin.cpp)
target_link_libraries(dokime-lifecycle-plugin PRIVATE Dokime::Client)
add_executable(dokime-lifecycle-journey integration/failed_initialization.cpp)
target_link_libraries(dokime-lifecycle-journey PRIVATE Dokime::Plugin Catch2::Catch2)
add_test(NAME failed_initialization
  COMMAND "${Python3_EXECUTABLE}" "${CMAKE_CURRENT_SOURCE_DIR}/support/run_plugin.py"
    $<TARGET_FILE:dokime-lifecycle-plugin> $<TARGET_FILE:dokime-lifecycle-journey> managed)
set_tests_properties(failed_initialization PROPERTIES TIMEOUT 240)
