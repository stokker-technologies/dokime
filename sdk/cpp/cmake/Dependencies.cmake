find_package(Protobuf CONFIG REQUIRED)
find_package(gRPC CONFIG REQUIRED)
find_package(OpenSSL REQUIRED)
include(FetchContent)
# The Pinned Library Owns Its Internal Wire Headers and Generated Context.
FetchContent_Declare(harmos
  URL https://github.com/stokker-technologies/harmos/archive/efc261fe9431bb3ad2fd3287ecb51a7f1d8da325.tar.gz
  URL_HASH SHA256=3a357640178f2da03d8ff77153f01a5f87f20ecc255e9b24f3cdc3f70765ff99
  SOURCE_SUBDIR sdk/cpp DOWNLOAD_EXTRACT_TIMESTAMP TRUE)
FetchContent_MakeAvailable(harmos)
