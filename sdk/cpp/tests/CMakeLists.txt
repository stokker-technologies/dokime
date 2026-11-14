add_executable(dokime-builders builders.cpp)
target_link_libraries(dokime-builders PRIVATE Dokime::Cpp)
add_test(NAME builders COMMAND dokime-builders)

add_executable(dokime-client-journey integration/client_reads_backend.cpp)
target_link_libraries(dokime-client-journey PRIVATE Dokime::Cpp)
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
  file(CONFIGURE OUTPUT "${CMAKE_CURRENT_BINARY_DIR}/conformance.cpp" CONTENT
    "#include <harmos/conformance.hpp>
int main(int argc, char** argv) { auto plugin = harmos::conformance::plugin(); if (argc == 3 && std::string(argv[1]) == \"--address\") plugin.serve(argv[2]); else plugin.serve(); }
" @ONLY)
  add_executable(dokime-conformance "${CMAKE_CURRENT_BINARY_DIR}/conformance.cpp")
  target_link_libraries(dokime-conformance PRIVATE Dokime::Cpp)
endif()
add_library(dokime_thermal_protocol STATIC)
protobuf_generate(TARGET dokime_thermal_protocol LANGUAGE cpp
  PROTOS "${CMAKE_CURRENT_SOURCE_DIR}/system/thermal.proto"
  IMPORT_DIRS "${CMAKE_CURRENT_SOURCE_DIR}/system" "${DOKIME_PROTOBUF_INCLUDE}"
  PROTOC_OUT_DIR "${DOKIME_GENERATED}" PROTOC_EXE "$<TARGET_FILE:protoc>")
target_link_libraries(dokime_thermal_protocol PUBLIC Dokime::Cpp)
add_executable(dokime-system-suite system/main.cpp)
target_link_libraries(dokime-system-suite PRIVATE dokime_thermal_protocol)
target_link_libraries(dokime-system-suite PRIVATE Dokime::Cpp)
if(WIN32)
  get_filename_component(compiler_directory "${CMAKE_CXX_COMPILER}" DIRECTORY)
  get_filename_component(protobuf_prefix "${DOKIME_PROTOBUF_INCLUDE}" DIRECTORY)
  set(package_bin "${protobuf_prefix}/bin")
  find_program(DOKIME_DUMPBIN dumpbin HINTS "${compiler_directory}" REQUIRED)
  string(CONFIGURE [=[
set(CMAKE_GET_RUNTIME_DEPENDENCIES_COMMAND "@DOKIME_DUMPBIN@")
if(EXISTS "$<TARGET_FILE:dokime-system-suite>")
  file(GET_RUNTIME_DEPENDENCIES
    EXECUTABLES "$<TARGET_FILE:dokime-system-suite>"
    DIRECTORIES "$<TARGET_FILE_DIR:dokime-system-suite>" "$<TARGET_FILE_DIR:libprotobuf>" "@package_bin@"
    PRE_EXCLUDE_REGEXES "api-ms-" "ext-ms-"
    POST_EXCLUDE_REGEXES ".*[Ss][Yy][Ss][Tt][Ee][Mm]32/.*"
    RESOLVED_DEPENDENCIES_VAR libraries
    UNRESOLVED_DEPENDENCIES_VAR missing)
  if(missing)
    message(FATAL_ERROR "Stage Fixture Runtime Libraries: Resolve ${missing}")
  endif()
  foreach(library IN LISTS libraries)
    file(COPY "${library}" DESTINATION "$<TARGET_FILE_DIR:dokime-system-suite>")
  endforeach()
endif()
]=] runtime_script @ONLY)
  file(GENERATE OUTPUT "${CMAKE_BINARY_DIR}/runtime-$<CONFIG>.cmake" CONTENT "${runtime_script}")
endif()
