# Generate Messages and Services with the Manifest's Matching Compiler and Runtime.
function(dokime_protocol target root output)
  file(MAKE_DIRECTORY "${output}")
  get_target_property(protobuf_includes protobuf::libprotobuf INTERFACE_INCLUDE_DIRECTORIES)
  protobuf_generate(TARGET ${target} PROTOS ${ARGN}
    IMPORT_DIRS "${root}" ${protobuf_includes} PROTOC_OUT_DIR "${output}"
    DEPENDENCIES protobuf::protoc "${CMAKE_CURRENT_FUNCTION_LIST_DIR}/../vcpkg.json" ${ARGN})
  protobuf_generate(TARGET ${target} LANGUAGE grpc PROTOS ${ARGN}
    IMPORT_DIRS "${root}" ${protobuf_includes} PROTOC_OUT_DIR "${output}"
    GENERATE_EXTENSIONS .grpc.pb.h .grpc.pb.cc
    PLUGIN "protoc-gen-grpc=$<TARGET_FILE:gRPC::grpc_cpp_plugin>"
    DEPENDENCIES protobuf::protoc gRPC::grpc_cpp_plugin "${CMAKE_CURRENT_FUNCTION_LIST_DIR}/../vcpkg.json" ${ARGN})
  target_include_directories(${target} SYSTEM PUBLIC
    $<BUILD_INTERFACE:${output}>
    $<INSTALL_INTERFACE:${CMAKE_INSTALL_INCLUDEDIR}/dokime/_wire>)
  target_compile_features(${target} PUBLIC cxx_std_20)
  # WinGDI Defines GetObject; Generated Storage Methods Must Keep Their Wire Name.
  target_compile_definitions(${target} PRIVATE "$<$<PLATFORM_ID:Windows>:NOGDI>")
  target_compile_options(${target} PRIVATE "$<$<CXX_COMPILER_ID:MSVC>:/bigobj>")
  target_link_libraries(${target} PUBLIC protobuf::libprotobuf gRPC::grpc++)
endfunction()
