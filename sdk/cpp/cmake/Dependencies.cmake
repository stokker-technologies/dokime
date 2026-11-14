find_package(Protobuf CONFIG REQUIRED)
find_package(gRPC CONFIG REQUIRED)
find_package(OpenSSL REQUIRED)
include(FetchContent)
# The Current Protocol Pin Predates Harmos's Installable CMake Package.
# Build Its Unmodified Sources Against Our One Manifest-Owned Dependency Graph.
FetchContent_Declare(harmos
  URL https://github.com/stokker-technologies/harmos/archive/81d32d3bc9ab3e2121746923712ec9508cee3ec7.tar.gz
  SOURCE_SUBDIR unused DOWNLOAD_EXTRACT_TIMESTAMP TRUE)
FetchContent_MakeAvailable(harmos)
set(harmos_sdk "${harmos_SOURCE_DIR}/sdk/cpp")
file(GLOB_RECURSE harmos_protocols CONFIGURE_DEPENDS
  "${harmos_SOURCE_DIR}/proto/harmos/*.proto")
list(FILTER harmos_protocols EXCLUDE REGEX "/guest\\.proto$")
add_library(harmos_cpp_protocol STATIC)
dokime_protocol(harmos_cpp_protocol "${harmos_SOURCE_DIR}/proto"
  "${CMAKE_CURRENT_BINARY_DIR}/g" CALLBACK_ONLY ${harmos_protocols})
add_library(harmos_cpp STATIC
  "${harmos_sdk}/src/catalog.cpp" "${harmos_sdk}/src/context.cpp"
  "${harmos_sdk}/src/server.cpp" "${harmos_sdk}/src/conformance.cpp")
add_library(Harmos::Cpp ALIAS harmos_cpp)
target_compile_features(harmos_cpp PUBLIC cxx_std_20)
target_include_directories(harmos_cpp PUBLIC
  $<BUILD_INTERFACE:${harmos_sdk}/include>
  $<INSTALL_INTERFACE:${CMAKE_INSTALL_INCLUDEDIR}>)
target_link_libraries(harmos_cpp PUBLIC harmos_cpp_protocol PRIVATE OpenSSL::Crypto)
