find_package(Protobuf CONFIG REQUIRED)
find_package(gRPC CONFIG REQUIRED)
find_package(OpenSSL REQUIRED)
include(FetchContent)
# The Pinned Library Owns Its Internal Wire Headers and Generated Context.
FetchContent_Declare(harmos
  URL https://github.com/stokker-technologies/harmos/archive/dae69cea0850ce8dd3bda4ec9d87c5c6c5ab3912.tar.gz
  URL_HASH SHA256=9e099ff46f9b04e466918a86460fb5b0b30baebc917de66aa14894bc88a09b3b
  SOURCE_SUBDIR sdk/cpp DOWNLOAD_EXTRACT_TIMESTAMP TRUE)
FetchContent_MakeAvailable(harmos)
