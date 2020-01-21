//===========================================================================
// Заготовка модуля XDBTDumperPlugin была сгенерирована специальным средством RAD разработки SA2-DE.
// Все права защищены. (2016)
//
// В данном файле представлен исходный текст основного заголовочного файла модуля.
//
//	В данном файле представлен паттерн интерфейса класса.
//  Описание класса:
//	Абстрактный класс, характеризующий данные для работы с модулем. Может представлять как входные, так и выходные параметры модуля
// ======================================= //
//       Параметры новой разработки.
// ======================================= //
// Имя комплекса:  GCAD  (gcd)
// Имя решения:    GCAD_2 (gc3)
// Имя проекта:    Client  (cli)
// Полный SID:     cligc3
// Время создания: 16:30:20
// Дата создания:  2015-10-28
// ======================================= //
// ============================================================================================ //
//                             Параметры ревизии(версии):
// ============================================================================================ //
// [prd]Period:                     02/2016
// [aut]Author:                     Антон Першин
// [did]DeveloperID:                ap
// [pid]ProblemID:                  00001
// [rvs = did.date.pid]Revision:    //#ap.2016-02-29.00001
// [dsc]Description:                Класс плагина для выгрузки xdbt-файлов.
// [ccm]CodeComment:                rvs.{[s]Start | [e]End | []}{[n]New | [o]Old | [d]Develop}
// ============================================================================================ //

//========================================================================
// ДОПОЛНИТЕЛЬНЫЕ ПОДКЛЮЧЕНИЯ (Типы и модули, необходимые для .h - файла)
//------------------------------------------------------------------------
#include "comaps_cls_GraphSolverWebHandler.h"
#include "comfrm_cls_Kernel.h"
#include "comfrm_unt_SQLTools.h"
#include "comfrm_unt_Logger.h"

#include <boost/bind.hpp>
#include <anymap.h>
#include <iniparser.h>
#include <fstream>

using namespace std;
using namespace ini;
using namespace com;
//========================================================================
void cls_XDBTDumperPlugin::execute(cls_AnyMap& p_input, ifc_ActionItem::tdf_onMessageClb p_MessCallback)
{
    m_callback = p_MessCallback;
    cls_AnyMap* answer = new cls_AnyMap();
    int err = 0;

    if(!p_input){

        err = ecEmptyData;
        
        COUT_LOG << "Solver execution didn`t done. Error code: " << err << endl;
        (*answer)[ "ERROR_CODE" ] = static_cast< int >(err);
        m_callback(boost::shared_ptr< cls_AnyMap >(answer)); 
        return;
    }
    else{
        string adotFile = p_input["ADOT_FNAME"];

        if(!adotFile){

            err = ecFileNotFound;

            
            COUT_LOG << "Solver execution didn`t done. Error code: " << err << endl;
            (*answer)[ "ERROR_CODE" ] = static_cast< int >(err);
            m_callback(boost::shared_ptr< cls_AnyMap >(answer)); 
            return;
        }
        else{

            ifstream file1("ADOT_FNAME"), file2("IN_FNAME");
            string test_read;
            if(!(getline(file1, test_read)){

                err = ecFileRead;

                
                COUT_LOG << "Solver execution didn`t done. Error code: " << err << endl;
                (*answer)[ "ERROR_CODE" ] = static_cast< int >(err);
                m_callback(boost::shared_ptr< cls_AnyMap >(answer)); 
                return;
            }
            else if(!(getline(file2, test_read)){

                err = ecFileRead;

                
                COUT_LOG << "Solver execution didn`t done. Error code: " << err << endl;
                (*answer)[ "ERROR_CODE" ] = static_cast< int >(err);
                m_callback(boost::shared_ptr< cls_AnyMap >(answer)); 
                return;
                
            }

            std::shared_ptr<com::graph::Node> adotNode = com::graph::loadFromADot(adotFile);

            if(!(adotNode->run(p_input["IN_FNAME":])){

                err = ecExecError;

                            
                COUT_LOG << "Solver execution didn`t done. Error code: " << err << endl;
                (*answer)[ "ERROR_CODE" ] = static_cast< int >(err);
                m_callback(boost::shared_ptr< cls_AnyMap >(answer)); 
                return;
            }
        }
    }

}


//========================================================================
