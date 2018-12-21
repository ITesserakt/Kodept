cmake_minimum_required(VERSION 2.8)

project(jugr)



#################################################################
### Объявление глобальных переменных, используемых при сборке ###
#################################################################

set(CMAKE_MODULE_PATH ${CMAKE_CURRENT_SOURCE_DIR}/cmake/modules)

# Добавляем поддержку C++14
include(CheckCXXCompilerFlag)
CHECK_CXX_COMPILER_FLAG("-std=c++14" COMPILER_SUPPORTS_CXX11)
CHECK_CXX_COMPILER_FLAG("-std=c++0x" COMPILER_SUPPORTS_CXX0X)
if(COMPILER_SUPPORTS_CXX11)
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -std=c++14")
elseif(COMPILER_SUPPORTS_CXX0X)
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -std=c++0x")
else()
        message(STATUS "The compiler ${CMAKE_CXX_COMPILER} has no C++14 support. Please use a different C++ compiler.")
endif()

# Указываем пути, в которых будет сохранятся результат сборки
if(CMAKE_BUILD_TYPE MATCHES "Debug")
    set(OUTPUT_DIRECTORY dbg CACHE STRING "")
else()
    set(OUTPUT_DIRECTORY rls CACHE STRING "")
endif()

set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_SOURCE_DIR}/${OUTPUT_DIRECTORY})
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_SOURCE_DIR}/${OUTPUT_DIRECTORY})
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_SOURCE_DIR}/${OUTPUT_DIRECTORY})

# В случае многоконфигурационной сборки указываем, в какие директории необходимо собирать
foreach(OUTPUTCONFIG ${CMAKE_CONFIGURATION_TYPES})
    string(TOUPPER ${OUTPUTCONFIG} OUTPUTCONFIG)
    set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY_${OUTPUTCONFIG} ${CMAKE_ARCHIVE_OUTPUT_DIRECTORY} )
    set(CMAKE_RUNTIME_OUTPUT_DIRECTORY_${OUTPUTCONFIG} ${CMAKE_RUNTIME_OUTPUT_DIRECTORY} )
    set(CMAKE_LIBRARY_OUTPUT_DIRECTORY_${OUTPUTCONFIG} ${CMAKE_LIBRARY_OUTPUT_DIRECTORY} )
endforeach( OUTPUTCONFIG CMAKE_CONFIGURATION_TYPES )

set(TESTED_LIBRARIES "" CACHE STRING "")


#######################
### Поиск библиотек ###
#######################

# Для Windows необходимо указать путь к boost.
# Предполагаем, что он определен к переменной среды BOOST
if(WIN32)
    set(BOOST_ROOT $ENV{BOOST})
endif()

find_package(Boost COMPONENTS filesystem system thread date_time REQUIRED)
if(Boost_FOUND)
    include_directories(${Boost_INCLUDE_DIRS})
endif()

include_directories($ENV{BOOST_PROCESS})

# Отключаем автоматическую линковку boost для Windows
if(WIN32)
    add_definitions(-DBOOST_ALL_NO_LIB)
endif()





set(COMSDK_LIBRARY ${COMSDK_INST}/lib/libcomsdk.so)

######################################################
### Дополнительные пути поиска заголовочных файлов ###
######################################################

include (CheckLibraryExists)

# Рекурсивно добавляем в пути поиска все директории исходных файлов решения

set(HEADER_DIRECTORIES )
file(GLOB_RECURSE ALL_HEADERS ${PROJECT_SOURCE_DIR}/dev/*.h)
foreach(HEADER_FILE ${ALL_HEADERS})
    get_filename_component(HEADER_DIR ${HEADER_FILE} PATH)
    set(HEADER_DIRECTORIES ${HEADER_DIRECTORIES} ${HEADER_DIR})
endforeach()

file(GLOB_RECURSE ALL_HEADERS ${PROJECT_SOURCE_DIR}/dev/*.hpp)
foreach(HEADER_FILE ${ALL_HEADERS})
    get_filename_component(HEADER_DIR ${HEADER_FILE} PATH)
    set(HEADER_DIRECTORIES ${HEADER_DIRECTORIES} ${HEADER_DIR})
endforeach()

list(REMOVE_DUPLICATES HEADER_DIRECTORIES)

include_directories(${HEADER_DIRECTORIES})
include_directories(${BOOST_PROCESS})

# подключаем зависимости

include_directories(${COMSDK_INST}/include)


############################################
### Добавление поддиректорий с проектами ###
############################################

add_subdirectory(dev)
add_subdirectory(tests/test_jugr)

#########################################################
### Копирование данных из ресурсов в выходной каталог ###
#########################################################
file(GLOB RESOURCES "res/*")
file(COPY ${RESOURCES} DESTINATION ${CMAKE_RUNTIME_OUTPUT_DIRECTORY})
