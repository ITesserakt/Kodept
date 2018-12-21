cmake_minimum_required(VERSION 2.8)
project(test_jugr)

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

target_link_libraries(${PROJECT_NAME} ${Boost_LIBRARIES}   ${COMSDK_LIBRARY} ${CMAKE_DL_LIBS})
