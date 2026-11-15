find_package(Protobuf CONFIG REQUIRED)
find_package(gRPC CONFIG REQUIRED)
find_package(OpenSSL REQUIRED)
include(FetchContent)
# The Pinned Library Owns Its Internal Wire Headers and Generated Context.
FetchContent_Declare(harmos
  URL https://github.com/stokker-technologies/harmos/archive/7181b5ed33c9d2be79c38d266b0a97d838b05ed6.tar.gz
  URL_HASH SHA256=443e82ef69925447277f500dffee56728d84556a81b5aa4c7c08a9eea852b015
  SOURCE_SUBDIR sdk/cpp DOWNLOAD_EXTRACT_TIMESTAMP TRUE)
FetchContent_MakeAvailable(harmos)
