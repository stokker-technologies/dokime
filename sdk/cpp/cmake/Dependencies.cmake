find_package(Protobuf CONFIG REQUIRED)
find_package(gRPC CONFIG REQUIRED)
find_package(OpenSSL REQUIRED)
include(FetchContent)
# The Pinned Library Owns Its Internal Wire Headers and Generated Context.
FetchContent_Declare(harmos
  URL https://github.com/stokker-technologies/harmos/archive/a8cee45c84b632a7a6987d94c911e0d76b6eed10.tar.gz
  URL_HASH SHA256=6b65b75d1e0a6fe897b7a95b4379bcc300b38cf56efa9d8bc70dca5143407f76
  SOURCE_SUBDIR sdk/cpp DOWNLOAD_EXTRACT_TIMESTAMP TRUE)
FetchContent_MakeAvailable(harmos)
