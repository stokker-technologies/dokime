cmake_minimum_required(VERSION 3.24)
project(DokimeDocuments LANGUAGES CXX)

include(FetchContent)
set(protobuf_BUILD_TESTS OFF CACHE BOOL "" FORCE)
set(protobuf_BUILD_PROTOC_BINARIES OFF CACHE BOOL "" FORCE)
set(protobuf_BUILD_LIBPROTOC OFF CACHE BOOL "" FORCE)
set(protobuf_MSVC_STATIC_RUNTIME OFF CACHE BOOL "" FORCE)
FetchContent_Declare(protobuf
  URL https://github.com/protocolbuffers/protobuf/releases/download/v36.2/protobuf-36.2.tar.gz
  URL_HASH SHA256=3d9642a662d10e68ebae5e53f14dcce5105684212d5078f8e0d47d1ab3ae6b64
  DOWNLOAD_EXTRACT_TIMESTAMP TRUE)
FetchContent_MakeAvailable(protobuf)

add_library(dokime-documents src/dokime/v1/documents.pb.cc src/dokime/v1/values.pb.cc)
target_compile_features(dokime-documents PUBLIC cxx_std_17)
target_include_directories(dokime-documents PUBLIC include)
target_link_libraries(dokime-documents PUBLIC protobuf::libprotobuf)

enable_testing()
add_executable(dokime-plot-builders tests/builders.cpp)
target_link_libraries(dokime-plot-builders PRIVATE dokime-documents)
add_test(NAME plot-builders COMMAND dokime-plot-builders)
