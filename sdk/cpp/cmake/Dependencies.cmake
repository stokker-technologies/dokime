find_package(Protobuf CONFIG REQUIRED)
find_package(gRPC CONFIG REQUIRED)
find_package(OpenSSL REQUIRED)
include(FetchContent)
# The Pinned Library Owns Its Internal Wire Headers and Generated Context.
FetchContent_Declare(harmos
  URL https://github.com/stokker-technologies/harmos/archive/6978e65f1d505c2fe805a34a2781e48ea6f30350.tar.gz
  URL_HASH SHA256=f806e18f53e7bab45f4c9e0ab79273b2860ff300c51f9904ed2700cccab99160
  SOURCE_SUBDIR sdk/cpp DOWNLOAD_EXTRACT_TIMESTAMP TRUE)
FetchContent_MakeAvailable(harmos)
