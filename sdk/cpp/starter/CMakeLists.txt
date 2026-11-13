cmake_minimum_required(VERSION 3.24)
project(DokimePlugin VERSION 0.1.0 LANGUAGES C CXX)
include(FetchContent)

# The Scaffold Writes Release Coordinates; Local Proof May Override the Source.
set(DOKIME_SDK_URL "" CACHE STRING "Versioned Dokime C++ Source Archive URL")
set(DOKIME_SDK_SHA256 "" CACHE STRING "Dokime C++ Source Archive SHA-256")
if(NOT FETCHCONTENT_SOURCE_DIR_DOKIME AND (NOT DOKIME_SDK_URL OR NOT DOKIME_SDK_SHA256))
  message(FATAL_ERROR "Set DOKIME_SDK_URL and DOKIME_SDK_SHA256 to the Matching Release Archive and Its Published SHA-256")
endif()
FetchContent_Declare(dokime URL "${DOKIME_SDK_URL}" URL_HASH "SHA256=${DOKIME_SDK_SHA256}"
  SOURCE_SUBDIR sdk/cpp DOWNLOAD_EXTRACT_TIMESTAMP TRUE)
FetchContent_MakeAvailable(dokime)
add_executable(dokime-plugin main.cpp)
target_link_libraries(dokime-plugin PRIVATE Dokime::Cpp)
