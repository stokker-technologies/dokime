find_package(Protobuf CONFIG REQUIRED)
find_package(gRPC CONFIG REQUIRED)
find_package(OpenSSL REQUIRED)
include(FetchContent)
# The Pinned Library Owns Its Internal Wire Headers and Generated Context.
FetchContent_Declare(harmos
  URL https://github.com/stokker-technologies/harmos/archive/640c73959d2a5ee65015afc617c0f3f2b4ca9f6c.tar.gz
  URL_HASH SHA256=6935b1d3aee58831b014afc36e99f9cf9e354abf9dd9d975e721da49a3684a0c
  SOURCE_SUBDIR sdk/cpp DOWNLOAD_EXTRACT_TIMESTAMP TRUE)
FetchContent_MakeAvailable(harmos)
