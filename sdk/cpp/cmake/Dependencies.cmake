find_package(Protobuf CONFIG REQUIRED)
find_package(gRPC CONFIG REQUIRED)
find_package(OpenSSL REQUIRED)
include(FetchContent)
# The Pinned Library Owns Its Internal Wire Headers and Generated Context.
FetchContent_Declare(harmos
  URL https://github.com/stokker-technologies/harmos/archive/aa9f5612efeea8f38bdd01c06b81f688ad11660f.tar.gz
  URL_HASH SHA256=9264fdc20ef9590458a3639d9ab45bedb9db4f24ecfc3549ff14bc9f2afc7eed
  SOURCE_SUBDIR sdk/cpp DOWNLOAD_EXTRACT_TIMESTAMP TRUE)
FetchContent_MakeAvailable(harmos)
