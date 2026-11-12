# Generate From the Protocol Root to Preserve Package Include Paths.
add_library(dokime_protocol STATIC)
file(GLOB protocols CONFIGURE_DEPENDS "${DOKIME_PROTO_ROOT}/dokime/v1/*.proto")
protobuf_generate(TARGET dokime_protocol LANGUAGE cpp
  PROTOS ${protocols} IMPORT_DIRS "${DOKIME_PROTO_ROOT}" "${DOKIME_PROTOBUF_INCLUDE}"
  DEPENDENCIES ${protocols} protoc
  PROTOC_OUT_DIR "${DOKIME_GENERATED}" PROTOC_EXE "$<TARGET_FILE:protoc>")
protobuf_generate(TARGET dokime_protocol LANGUAGE grpc
  PROTOS ${protocols} IMPORT_DIRS "${DOKIME_PROTO_ROOT}" "${DOKIME_PROTOBUF_INCLUDE}"
  DEPENDENCIES ${protocols} protoc grpc_cpp_plugin
  PROTOC_OUT_DIR "${DOKIME_GENERATED}" PROTOC_EXE "$<TARGET_FILE:protoc>"
  GENERATE_EXTENSIONS .grpc.pb.h .grpc.pb.cc
  PLUGIN "protoc-gen-grpc=$<TARGET_FILE:grpc_cpp_plugin>")
