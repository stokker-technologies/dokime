install(TARGETS dokime_cpp dokime_protocol harmos_cpp harmos_cpp_protocol
  EXPORT DokimeTargets ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR})
install(DIRECTORY include/ "${harmos_sdk}/include/" DESTINATION ${CMAKE_INSTALL_INCLUDEDIR})
foreach(protocol_target IN ITEMS dokime_protocol harmos_cpp_protocol)
  get_target_property(generated_sources ${protocol_target} SOURCES)
  foreach(header IN LISTS generated_sources)
    if(header MATCHES "\\.h$")
      file(RELATIVE_PATH relative "${DOKIME_GENERATED}" "${header}")
      get_filename_component(directory "${relative}" DIRECTORY)
      install(FILES "${header}" DESTINATION "${CMAKE_INSTALL_INCLUDEDIR}/${directory}")
    endif()
  endforeach()
endforeach()
configure_package_config_file(cmake/DokimeConfig.cmake.in
  "${CMAKE_CURRENT_BINARY_DIR}/DokimeConfig.cmake"
  INSTALL_DESTINATION "${CMAKE_INSTALL_LIBDIR}/cmake/Dokime")
write_basic_package_version_file("${CMAKE_CURRENT_BINARY_DIR}/DokimeConfigVersion.cmake"
  VERSION ${PROJECT_VERSION} COMPATIBILITY SameMajorVersion)
install(EXPORT DokimeTargets NAMESPACE Dokime:: DESTINATION "${CMAKE_INSTALL_LIBDIR}/cmake/Dokime")
install(FILES "${CMAKE_CURRENT_BINARY_DIR}/DokimeConfig.cmake"
  "${CMAKE_CURRENT_BINARY_DIR}/DokimeConfigVersion.cmake"
  DESTINATION "${CMAKE_INSTALL_LIBDIR}/cmake/Dokime")
