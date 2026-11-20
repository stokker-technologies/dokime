# Generate Messages, or Client Stubs with LANGUAGE grpc, with the Manifest's Matching Compiler and Runtime.
function(dokime_generate variable language)
  file(MAKE_DIRECTORY "${DOKIME_GENERATED}")
  get_target_property(protobuf_includes protobuf::libprotobuf INTERFACE_INCLUDE_DIRECTORIES)
  set(arguments LANGUAGE cpp)
  set(dependencies protobuf::protoc)
  if(language STREQUAL "grpc")
    set(arguments LANGUAGE grpc GENERATE_EXTENSIONS .grpc.pb.h .grpc.pb.cc
      PLUGIN "protoc-gen-grpc=$<TARGET_FILE:gRPC::grpc_cpp_plugin>")
    list(APPEND dependencies gRPC::grpc_cpp_plugin)
  endif()
  protobuf_generate(OUT_VAR generated ${arguments} PROTOS ${ARGN}
    IMPORT_DIRS "${DOKIME_PROTO_ROOT}" ${protobuf_includes} PROTOC_OUT_DIR "${DOKIME_GENERATED}"
    DEPENDENCIES ${dependencies} "${CMAKE_CURRENT_FUNCTION_LIST_DIR}/../vcpkg.json" ${ARGN})
  set(${variable} ${generated} PARENT_SCOPE)
endfunction()

# Compile Generated Sources With Their Wire Include Root.
function(dokime_protocol target)
  add_library(${target} STATIC ${ARGN})
  target_include_directories(${target} SYSTEM PUBLIC
    $<BUILD_INTERFACE:${DOKIME_GENERATED}>
    $<INSTALL_INTERFACE:${CMAKE_INSTALL_INCLUDEDIR}/dokime/_wire>)
  target_compile_features(${target} PUBLIC cxx_std_20)
  # WinGDI Defines GetObject; Generated Storage Methods Must Keep Their Wire Name.
  target_compile_definitions(${target} PRIVATE "$<$<PLATFORM_ID:Windows>:NOGDI>")
  target_compile_options(${target} PRIVATE "$<$<CXX_COMPILER_ID:MSVC>:/bigobj>")
endfunction()
