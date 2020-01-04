"""
Схема
|---
    |   
A   |---/$GIT_REP_PATH/*.* *.pdf *.doc *.so ...                

B   |---/mnt/sda1_6tb/owncloud_data/$DB_USER/files/

cpx - SID комплекса - атрибуты выгружаются из GCDDB.sys.cmplx
sln - SID решения - атрибуты выгружаются из GCDDB.sys.solun
tml - SID шаблона (типа документа) - атрибуты выгружаются из GCDDB.gen.tmpls
ext = (.pdf|.doc|.so|...)

Алгоритм работы программы:
0. Сканируем репозиторий на предмет файлов определённых типов.
1. Анализируем имена файлов заданных типов на соответствие шаблону: cpxsln_tml_Name.ext.
2. Если имя файла соответствует шаблону, то формируем для его хранения структуру каталогов и копируем этот файл в нее.

"""

import sys
import psycopg2
import psycopg2.extras
import string
import os
import shutil
import re
import getpass
import pwd
import grp
import filecmp

# Поиск файлов, заданных типов, по заданному адресу.
def FindFilesByPathAndExtentions(path):
    ExtentionsToFind = []
    find_files = []
    i = 8
    while i < len (sys.argv):
        ExtentionsToFind.append(sys.argv[i])
        i += 1
    for root, dirs, files in os.walk(path):
        for file in files:
            for ext in ExtentionsToFind:
                if file.endswith(ext):
                    find_files.append(os.path.join(root, file))
    return find_files

#создание кортежей расшифровки имен систем и подсистем
def load_CPX_SLN_TML_Attrs(sys_argv):
    try:
        conn = psycopg2.connect(dbname=sys_argv[1], user=sys_argv[2], password=sys_argv[3], host=sys_argv[4], port=sys_argv[5])
    except psycopg2.Error as err:
        print("Connection error: {}".format(err))
    cur = conn.cursor()
    try:
#        #Определяем имена систем из БД, для которых заданы rlvnc и dirna
        cur.execute("SELECT a.cpxid, a.rlvnc, a.dirna from sys.cmplx as a where a.rlvnc<>'' and a.dirna<>'';")
        sys_rows = cur.fetchall()   
        #Определяем имена подсистем из БД, для которых заданы rlvnc и dirna
#        cur.execute("SELECT b.cpxid, b.rlvnc, b.dirna, a.slnid, a.rlvnc, a.dirna from sys.solun as a, sys.cmplx as b where a.cpxid=b.cpxid and a.rlvnc<>'' and a.dirna<>'' and b.rlvnc<>'' and b.dirna<>'';")
        cur.execute("SELECT a.slnid, a.rlvnc, a.dirna from sys.solun as a where a.rlvnc<>'' and a.dirna<>'';")
        subsys_rows = cur.fetchall()
        #Определяем имена шаблонов из БД, для которых заданы rlvnc и foldr
        cur.execute("SELECT a.tmlid, a.rlvnc, a.foldr from gen.tmpls as a where a.rlvnc<>'' and a.foldr<>'';")
        template_rows = cur.fetchall()
    except psycopg2.Error as err:
        print("Execute: {}".format(err))    
    return sys_rows, subsys_rows, template_rows

def createDestDir(path):
    # Проверяем не был ли ранее создан каталог
    if not os.path.isdir(path):
        try:
            print(getpass.getuser())
            os.mkdir(path)
            print("Created directory: ",  path)
        except OSError as err:
            print("Unable to create directory: ",  path,"\n Description: {}".format(err))
            return False
    return True

# source_filename - имя, анализируемого файла
# dest_path - адрес корневой папки-назначения 
# p_CPX_Attrs - массив систем с атрибутами
# p_SLN_Attrs - массив подсистем с атрибутами
# p_TML_Attrs - массив типов документов (шаблонов) с атрибутами
# dir_level = 1 - первый уровень, уровень систем 
# dir_level = 2 - второй уровень, уровень подсистем 
# dir_level = 3 - третий уровень, уровень типов документов 
#
# returns - полное имя файла в целевой папке
def makeDirStructureForFile(source_filename, dest_path, CPX_Attrs, SLN_Attrs, TML_Attrs):

    newTMLDirName = ""
    dirs_to_chown = []

    # Флаг корректности операции
    flag = False 
    # Пытаемся найти среди предварительно подготовленных объектов (систем, подсистем, типов документов (шаблонов)), для которых указаны rlvnc, dirna, такой,
    # для которого совпадёт анализуемый соответствующий атрибут имени файла
    for cpx_object in CPX_Attrs:
        if cpx_object[0]==source_filename[0:3]:
            for sln_object in SLN_Attrs:
                if sln_object[0]==source_filename[3:6]:
                    #print(cpx_object[0], " ", sln_object[0], " ", source_filename[0:3], " ", source_filename[3:6], " ", source_filename)
                    # cpx_object[0] - sys.cmplx.cpxid 
                    # cpx_object[1] - sys.cmplx.rlvnc 
                    # cpx_object[2] - sys.cmplx.dirna 
                    # sln_object[0] - sys.solun.slnid 
                    # sln_object[1] - sys.solun.rlvnc
                    # sln_object[2] - sys.solun.dirna
                    # 
                    # Создаём каталог 1 уровня - уровень системы
                    newCPXDirName = cpx_object[1].rstrip() + " - " + cpx_object[2].rstrip()
                    newCPXDirName = os.path.join(dest_path, newCPXDirName)
                    #print ("newCPXDirName=",newCPXDirName)
                    if createDestDir(newCPXDirName):
                        dirs_to_chown.append(newCPXDirName)
                        #print(dirs_to_chown)
                        # Создаём каталог 2 уровня - уровень подсистемы
                        newSLNDirName = sln_object[1].rstrip() + " - " + sln_object[2].rstrip()
                        newSLNDirName = os.path.join(newCPXDirName, newSLNDirName)
                        #print ("newSLNDirName=",newSLNDirName)
                        if createDestDir(newSLNDirName):
                            for tml_object in TML_Attrs:
                                #print(tml_object[0].lower(), " ", source_filename[7:10], " ", source_filename)
                                if tml_object[0].lower()==source_filename[7:10]:
                                    # Создаём каталог 3 уровня - уровень типа документа
                                    newTMLDirName = tml_object[1].rstrip() + " - " + tml_object[2].rstrip()
                                    newTMLDirName = os.path.join(newSLNDirName, newTMLDirName)
                                    #print ("newTMLDirName=",newTMLDirName)
                                    if createDestDir(newTMLDirName):
                                        flag = True
                                        break
                if flag:
                    break
        if flag:
            break
    if flag:
        destFilename = source_filename[11:]
        destFilePath = os.path.join(newTMLDirName, destFilename)
    else:
        destFilePath = ""
    return flag, destFilePath, dirs_to_chown

#Анализ имени файла и копирование в целевое расположение
def parseFilenameAndCopy(sourceFilePath, destRootPath, p_CPX_Attrs, p_SLN_Attrs, p_TML_Attrs):
    
    res = False

    sourceFilename = os.path.basename(sourceFilePath)
    # Проверяем формат имени анализируемого файла
    if not re.match(r'^\w{3}\w{3}_\w{3}_\w*', os.path.basename(sourceFilePath)):
        print("Non-standard filename:", os.path.basename(sourceFilePath))
        return res, []

    # Формируем целевую структуру каталогов для файла sourceFilename
    res, destFilePath, dirs_to_chown = makeDirStructureForFile(sourceFilename, destRootPath, p_CPX_Attrs, p_SLN_Attrs, p_TML_Attrs)

    # Проверяем не одинаковые ли файлы перед копированием. Если одинаковые, то копировать не следует.
    if os.path.isfile(destFilePath) and filecmp.cmp(sourceFilePath, destFilePath): 
        print("Копирование отменено, файл:", sourceFilename, " уже существует по адресу:", destFilePath, "\n Файлы одинаковы.")
        return False, [] 

    #print (res)
    if res:
        # копируем файл в целевое местоположение с изменением имени
        try:
            shutil.copy(sourceFilePath, destFilePath)
        except OSError as err:
            print("Unable to copy file:", sourceFilename, " to:", destFilePath, "\n Description: {}".format(err))
            return False
        print("File:", sourceFilename, " copied to:", destFilePath)    
 
    return res, dirs_to_chown

#============================================================
#================= Основной код программы ===================
#============================================================

# Поиск файлов
found_files = FindFilesByPathAndExtentions(sys.argv[6])

#Запоминаем полные пути ко всем найденным файлам
FullPaths = []
[FullPaths.append(os.path.abspath(ConsideredFile)) for ConsideredFile in found_files]

print("Found files:")
for f in found_files:
	print(f)
print('\n')

#создание кортежей имен каталогов на основании данных из БД
CPX_Attrs, SLN_Attrs, TML_Attrs = load_CPX_SLN_TML_Attrs(sys.argv)

print("Systems:")
for f in CPX_Attrs:
    if f[2] is not None:
    	print(f)
print('\n')

print("Subsystems:")
for f in SLN_Attrs:
    if f[2] is not None:
    	print(f)
print('\n')

print("Templates:")
for f in TML_Attrs:
    if f[2] is not None:
    	print(f)
print('\n')

#Анализ имен и размещение документов
dirs_to_chown = []
for FullFilename in FullPaths:
    #print(FullFilename)
    res, dirs = parseFilenameAndCopy(FullFilename, sys.argv[7], CPX_Attrs, SLN_Attrs, TML_Attrs)
    if res==True:
        for dire in dirs:
            dirs_to_chown.append(dire)

dirs_to_chown = list(dict.fromkeys(dirs_to_chown))

print("Dirs to chown:")
for d in dirs_to_chown:
    print(d)

for chowndir in dirs_to_chown:
    for c_dirpath, c_child_dirs, c_files in os.walk(chowndir):
        #for pdir in c_dirs:
            #os.chown(os.path.join(c_root, pdir), uid, gid)
        shutil.chown(c_dirpath, -1, group="apache")
        print("CHOWNed Dir:",c_dirpath)
        for file in c_files:
            shutil.chown(os.path.join(c_dirpath, file), -1, group="apache")
            print("CHOWNed File:",os.path.join(c_dirpath, file))
