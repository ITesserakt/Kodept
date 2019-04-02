//========================================================================
#ifndef comfrm_cls_XDBTDumperPluginH
#define comfrm_cls_XDBTDumperPluginH
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
#include "comfrm_ifc_ExtendedPlugin.h"
#include "comfrm_ifc_ActionItem.h"
#include "comfrm_cls_SQLTable.h"

#include <boost/bind.hpp>
//========================================================================
class cpl_GraphSolverWebHandler : public ifc_ExtendedPlugin
{
public:

    enum enu_ErrorCode
        {
            ecSuccess = 0,
            ecEmptyData = 100,
            ecFileNotFound,
            ecExecError = 200,
        };

    virtual ~cpl_GraphSolverWebHandler(){}

    /*!
    Возвращает уникальный строковый идентификатор плагина
    \return Строковый идентификатор
    */
    virtual std::string getPluginSID() const {return "GRAPH_SOLVER";}

    /*!
    Возвращает имя плагина
    \return Имя плагина
    */
    virtual std::string getPluginName() const {return "GRAPH_SOLVER";}

    /*!
    Возвращает тип плагина
    \return Тип плагина
    */
    virtual std::string getPluginType() const {return "COMAPS";}

    /*!
    Деинициализирует плагин. Выполняется при удалении плагина из системы
    */
    virtual void destroyPlugin(){return;}

    /*!
    Запускает плагин, передавая на вход AnyMap с входными данными
    \param[in] p_input Входные данные
    */
    virtual void execute(cls_AnyMap& p_input, ifc_ActionItem::tdf_onMessageClb p_MessCallback);

    void onTableReceived(boost::shared_ptr< cls_SQLTable > p_table);

private:
    ifc_ActionItem::tdf_onMessageClb m_callback;
};

//========================================================================
#endif
//========================================================================
