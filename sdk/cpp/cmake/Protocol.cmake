# Generate Messages and Services with the Manifest's Matching Compiler and Runtime.
function(dokime_protocol target root output)
  cmake_parse_arguments(PROTOCOL "CALLBACK_ONLY" "" "" ${ARGN})
  file(MAKE_DIRECTORY "${output}")
  get_target_property(protobuf_includes protobuf::libprotobuf INTERFACE_INCLUDE_DIRECTORIES)
  protobuf_generate(TARGET ${target} PROTOS ${PROTOCOL_UNPARSED_ARGUMENTS}
    IMPORT_DIRS "${root}" ${protobuf_includes} PROTOC_OUT_DIR "${output}"
    DEPENDENCIES protobuf::protoc ${PROTOCOL_UNPARSED_ARGUMENTS})
  set(service_protocols ${PROTOCOL_UNPARSED_ARGUMENTS})
  set(server_options)
  if(PROTOCOL_CALLBACK_ONLY)
    set(server_options "allow_sync_server_api=false,allow_cq_api=false")
    # Context Contracts Are Carrier Payloads, Without Independent gRPC Services.
    list(FILTER service_protocols EXCLUDE REGEX "/harmos/context/")
  endif()
  protobuf_generate(TARGET ${target} LANGUAGE grpc PROTOS ${service_protocols}
    IMPORT_DIRS "${root}" ${protobuf_includes} PROTOC_OUT_DIR "${output}"
    GENERATE_EXTENSIONS .grpc.pb.h .grpc.pb.cc
    PLUGIN "protoc-gen-grpc=$<TARGET_FILE:gRPC::grpc_cpp_plugin>"
    PLUGIN_OPTIONS "${server_options}"
    DEPENDENCIES protobuf::protoc gRPC::grpc_cpp_plugin ${PROTOCOL_UNPARSED_ARGUMENTS})
  target_include_directories(${target} SYSTEM PUBLIC
    $<BUILD_INTERFACE:${output}>
    $<INSTALL_INTERFACE:${CMAKE_INSTALL_INCLUDEDIR}>)
  target_compile_features(${target} PUBLIC cxx_std_20)
  # WinGDI Defines GetObject; Generated Storage Methods Must Keep Their Wire Name.
  target_compile_definitions(${target} PRIVATE "$<$<PLATFORM_ID:Windows>:NOGDI>")
  target_compile_options(${target} PRIVATE "$<$<CXX_COMPILER_ID:MSVC>:/bigobj>")
  target_link_libraries(${target} PUBLIC protobuf::libprotobuf gRPC::grpc++)
endfunction()
