install(TARGETS dokime_plugin dokime_client dokime_plugin_protocol dokime_client_protocol
  EXPORT DokimeTargets ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR})
install(DIRECTORY include/ DESTINATION ${CMAKE_INSTALL_INCLUDEDIR})
foreach(header IN LISTS plugin_messages client_messages client_services)
  if(header MATCHES "\\.h$")
    file(RELATIVE_PATH relative "${DOKIME_GENERATED}" "${header}")
    get_filename_component(directory "${relative}" DIRECTORY)
    install(FILES "${header}" DESTINATION "${CMAKE_INSTALL_INCLUDEDIR}/dokime/_wire/${directory}")
  endif()
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
