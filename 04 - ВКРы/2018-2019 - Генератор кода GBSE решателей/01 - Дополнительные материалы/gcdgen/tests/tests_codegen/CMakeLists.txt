cmake_minimum_required(VERSION 3.10.2)

project(tests_codegen)

######################
### Исходные файлы ###
######################

set(SOURCES
        main.cpp
        )

########################################
### Добавление финальной цели сборки ###
########################################
add_executable(${PROJECT_NAME} ${SOURCES})

message("${COMSDK_LIBRARY}")

target_link_libraries(${PROJECT_NAME} ${Boost_LIBRARIES} ${COMSDK_LIBRARY} ${CMAKE_DL_LIBS})
