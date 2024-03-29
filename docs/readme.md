# Использозование шаблона КП, НИРС, ВКР

Шаблон позволяет собирать расчетно-пояснительные записки в рамках курсовых проектов (КП), курсовых работ (КР), научно-исследовательских работ студентов (НИРС), выпускных квалификационных работ (ВКР) бакалавра, а также научно-квалификационных работ (НКР) магистра.

## Лицензия

* Шаблон предоставляется на условиях лицензии CC BY-SA 4.0

## Особенности сборки

Для сборки РПЗ соответствующего типа необходимо указать значение соответствующего параметра в файле `constants_id.tex`

```latex
\newcommand{\doctypesid}{nirs} % vkr (выпускная квалификационная работа) / kp (курсовой проект) / kr (курсовая работа) / nirs (научно-исследовательская работа студента) / nkr (научно-квалификационная работа)
```

### Особенности сборки ВКР бакалавра или магистра

* Для прохождения проверки с помощью ПО `TestVkr` в исходниках следует закомментировать следующие строки в ряде файлов. Указанный ниже материал необходим для добавления водяных знаков, которые ПО `TestVkr` воспринимает как ошибки. Вместе с тем этот материал, помимо прочих средств, обеспечивает защиту авторских прав обучающегося.
* Файл `preamble_common.tex`:

```latex
    \usepackage[hpos=0.98\paperwidth,
            vpos=0.7\paperwidth,
            angle=90]{draftwatermark}
    \SetWatermarkText{\authorSIDright}
    \SetWatermarkColor[gray]{0.1}
    \SetWatermarkFontSize{0.2cm}
    \SetWatermarkAngle{90}
    \SetWatermarkHorCenter{20cm}
```

* Файл `title_common.tex`:

```latex
\begin{textblock}{1}(0,0)
\rotatebox{90}{\textcolor{gray!20.}{МГТУ им. Н.Э.Баумана, кафедра <<Системы автоматизированного проектирования>> (РК-6), шаблон RPT (размещение sa2tml)}}
\end{textblock}
```

## Возможные недочёты

> Файл `G7_32.cls`
```latex
% возможно, что эту строку следует раскомментировать
%\RequirePackage[left=30mm,right=10mm,top=20mm,bottom=20mm,headsep=0pt]{geometry}
% сейчас используется эта
\RequirePackage[left=20mm,right=10mm,top=20mm,bottom=25mm,headsep=0pt]{geometry}

```

## Контакты

* **Соколов Александр Павлович** (доцент кафедры САПР, МГТУ им. Н.Э. Баумана)
  * моб.: `+7(916)9093342`
  * e-mail: alsokolo@bmstu.ru
