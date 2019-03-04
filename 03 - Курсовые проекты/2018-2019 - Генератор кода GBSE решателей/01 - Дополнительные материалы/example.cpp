#include <anymap.h>
//---------------------------------------------------
//  Определяем стандартную сигнатуру функции-обратчика
typedef int processorFuncType(AnyMap& );
//---------------------------------------------------
//  Определяем стандартную сигнатуру функции-предиката
typedef bool predicateFuncType(const AnyMap& );
//---------------------------------------------------
template<processorFuncType* tf, predicateFuncType* tp>
int F(AnyMap& p_m)
{
	return (tp(p_m))?tf(p_m):tp(p_m);
}
//---------------------------------------------------


int main()
{
	// Создаём объект общих данных на основе класса AnyMap
	AnyMap m("input.aini");
	
	//  Загружаем все библиотеки решателя в адресное пространство текущего процесса
	HMODULE lib_@lib_name_i@ = LoadLibrary(L"@lib_name_i@");
	HMODULE lib_@lib_name_i@ = LoadLibrary(L"@lib_name_i@");
	...

	// Осуществляем поиск функций-обработчиков и предикатов в соответствующих библиотеках 
	processorFuncType *proc_@processor_i@ = (processorFuncType*)GetProcAddress(lib_@lib_name_i@, "@processor_i@");
	processorFuncType *proc_@processor_i@ = (processorFuncType*)GetProcAddress(lib_@lib_name_i@, "@processor_i@");
	...
	
	predicateFuncType *pred_@predicate_i@ = (predicateFuncType*)GetProcAddress(lib_@lib_name_i@, "@predicate_i@");
	predicateFuncType *pred_@predicate_i@ = (predicateFuncType*)GetProcAddress(lib_@lib_name_i@, "@predicate_i@");
	...
	
	res = F<proc_@processor_i@,pred_@predicate_i@>(m); if (res != 0) return res;
	res = F<proc_@processor_i@,pred_@predicate_i@>(m); if (res != 0) return res;
	res = F<proc_@processor_i@,pred_@predicate_i@>(m); if (res != 0) return res;

	do {
	res = F<proc_@processor_i@,pred_@predicate_i@>(m); if (res != 0) return res;
	...
	}
	while (p6(m));

	res = F<proc_@processor_i@,pred_@predicate_i@>(m); if (res != 0) return res;

	return 0;
}