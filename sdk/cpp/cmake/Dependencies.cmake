find_package(Protobuf CONFIG REQUIRED)
find_package(gRPC CONFIG REQUIRED)
find_package(OpenSSL REQUIRED)
include(FetchContent)
# The Pinned Library Owns Its Internal Wire Headers and Generated Context.
FetchContent_Declare(harmos
  URL https://github.com/stokker-technologies/harmos/archive/5d601dd320edb0fc69131dc2851d789c4dd1e4c0.tar.gz
  URL_HASH SHA256=30047f5b96b3c41cb59e853f39c6e23d52fc7edab0975db2e9f49b66542ba5a2
  SOURCE_SUBDIR sdk/cpp DOWNLOAD_EXTRACT_TIMESTAMP TRUE)
FetchContent_MakeAvailable(harmos)
