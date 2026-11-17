find_package(Protobuf CONFIG REQUIRED)
find_package(gRPC CONFIG REQUIRED)
find_package(OpenSSL REQUIRED)
include(FetchContent)
# The Pinned Library Owns Its Internal Wire Headers and Generated Context.
FetchContent_Declare(harmos
  URL https://github.com/stokker-technologies/harmos/archive/fa52a0e6619803c824af5112eb28b9a85430b359.tar.gz
  URL_HASH SHA256=271b05f47fec1ba5023f74b6ada54e9273e0721c2e58099228eb99cc09f908cb
  SOURCE_SUBDIR sdk/cpp DOWNLOAD_EXTRACT_TIMESTAMP TRUE)
FetchContent_MakeAvailable(harmos)
