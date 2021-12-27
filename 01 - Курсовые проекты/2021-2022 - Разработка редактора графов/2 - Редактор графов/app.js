//------В этом блоке вычисляется оптимальная ширина и высота холста
// Ширина левой панели
const leftSidebarWidth = parseInt(
  window
    .getComputedStyle(document.querySelector(".left-sidebar"))
    .width.slice(0, -2)
);
// Ширина правой панели
const rightSidebarWidth = parseInt(
  window
    .getComputedStyle(document.querySelector(".right-sidebar"))
    .width.slice(0, -2)
);
// Высота нижней панели
const bottomSidebarHeight = parseInt(
  window
    .getComputedStyle(document.querySelector(".information-console"))
    .height.slice(0, -2)
);
// Высота body
const bodyHeight = parseInt(
  window.getComputedStyle(document.querySelector("body")).height.slice(0, -2)
);

//------Константы которые используется в приложении
const svgCanvasHeight = bodyHeight - bottomSidebarHeight; // Оптимальная высота холста в пикселях
const svgCanvasWidth =
  window.screen.width - leftSidebarWidth - rightSidebarWidth; // Оптимальная ширина холста в пикселях
const maxLabelLength = 5; // Максимальная длина текста для метки

//------Создаем пустой холст на странице
const svg = d3
  .select("svg")
  .attr("width", svgCanvasWidth)
  .attr("height", svgCanvasHeight)
  .attr("class", "svg-container");

//------Все объекты, которые используются на странице
const svgCanvas = document.getElementsByClassName("svg-container")[0]; // Холст
const SetEdgeButton = document.getElementById("set_edge_button"); // Кнопка "Добавить ребро"
const SetVertexButton = document.getElementById("set_vertex_button"); // Кнопка "Добавить вершину"
const DeleteVertexButton = document.getElementById("remove_vertex_button"); // Кнопка "Удалить вершину"
const DeleteEdgeButton = document.getElementById("remove_edge_button"); // Кнопка "Удалить ребро"
const ClearCanvasButton = document.getElementById("clear_canvas_button"); // Кнопка "Очистить холст"
const ExportADOTButton = document.getElementById("export_adot_button"); // Кнопка "Экспортировать в формат aDOT"
const ImportADOTButton = document.getElementById("input_adot"); // Кнопка "Импортировать из формата aDOT"
const FindCyclesButton = document.getElementById("find_cycles_button"); // Кнопка найти циклы
const RightSidebarBlock = document.getElementsByClassName(
  "right-sidebar-block"
)[0]; // Объект блока правого сайдбара, необходим для того, чтобы скрывать или показывать пользователю поля ввода в
// течение работы с приложением

//---Объекты которые рендерятся в процессе работы с приложением
const RightSidebarConfirmButton = document.getElementById(
  "right-sidebar__button_id"
); // Кнопка в правом сайдбаре
const InformationConsoleMessage = document.getElementsByClassName(
  "information-console__message"
)[0]; // Текст внутри нижнего сайдбара

//------Класс который реализует всю бизнес-логику работы в графом
class Graph {
  //------Конструктор класса - объявление и инициализация необходимых переменных-членов класса
  constructor() {
    this.vertices = {}; // Объект для хранения всей информации о вершинах (позиция на холсте, метка вершину, ребра
    // входящие и выходящие из вершины и т.д.)
    this.vertexID = 0; // ID вершины, после построения вершины - инкрементируеться, необходим для задания уникального
    // ID для построенной вершины
    this.edgeID = 0; // ID ребра, после построения ребра - инкрементируеться, необходим для задания уникального
    // ID для построенного ребра
    this.createdVerticesPositions = []; // Массив, который хранит информацию о позициях построенных вершин,
    // необходим для того, чтобы не допускать построение вершин в непосредственной близости или друг на друге
    this.createdVerticesLabels = new Set(); // Коллекция уникальных значений, необходима для сохранения меток
    // построенных вершин, с целью предотвращения одинаковых меток у нескольких вершин
    this.predicates = {}; // Объект для хранения информации о предикатах в графе (aDOT)
    this.functions = {}; // Объект для хранения информации о функциях в графе (aDOT)
    this.edges = {}; // Объект для хранения информации об edge (aDOT)

    this.radius = 30; // Радиус вершины, 30px - по умолчанию
    this.borderWidth = 5; // Толщина границы вершины, 5px - по умолчанию
    this.arrowheadSize = 10; // Величина, которая характеризует размер стрелки, 10 - по умолчанию, чем больше величина
    // тем больше размер стрелки

    this.pathShortest = [];
    this.pathIncludesVertex = [];
    this.pathShortestLength = 10000;
    this.pathIncludesVertexLength = 10000;
  }

  /**
   * Публичная функция-член класса, реализующая бизнес-логику по созданию вершины
   *
   * @param positionX - позиция вершины по координате X
   * @param positionY - позиция вершины по координате Y
   * @returns {boolean} - успешно/неуспешно создана
   */
  AddVertex(positionX, positionY) {
    // Проверяем корректность позиции новой вершины
    if (!this.#isVertexPositionValid(positionX, positionY)) {
      // Устанавливаем сообщение в консоли (нижний сайдбар)
      setInformationConsoleMessage(
        `Вы не можете создать вершину в этом месте, она находится слишком близко к другой вершине`
      );
      // В процессе создания вершины, произошла ошибка -> return false
      return false;
    }

    // ID для новой вершины
    const id = "vertex" + ++this.vertexID;

    // Создаем новую вершину на холсте, из функции получаем объект типа d3.path() - мнимая линия внутри вершины
    //по которой будет строиться текст
    const path = CanvasCreateVertex(
      positionX,
      positionY,
      id,
      this.radius,
      this.borderWidth
    );
    CanvasChangeBorderColor([id], "#B92808");

    // Сохраняем позицию созданной вершины в массиве createdVerticesPositions (для чего он нужен описано в
    // комментариях в конструкторе класса)
    this.createdVerticesPositions.push([id, positionX, positionY]);

    // Блокируем любые действия на странице, необходимо для того, чтобы пользователь ввел текст метки и не
    // смог выбрать другое действие, поскольку построение вершины не завершено
    actionsDisabled = true;

    // Если отсутствует поле ввода для метки вершины, рендерим его в правом сайдбаре
    if (document.getElementById("right-sidebar__input__id") === null) {
      renderVertexInput();
    }
    // Включаем видимость правого блока display: none -> display: flex
    RightSidebarBlock.style.display = "flex";
    // Устанавливаем сообщение в консоли (нижний сайдбар)
    setInformationConsoleMessage(
      `Вы успешно создали вершину, теперь введите метку для вершины в поле на правой панели`
    );

    /**
     * Обработчик нажатия на кнопку подтверждения ввода в правом сайдбаре
     */
    const setLabelHandler = () => {
      // Из поля ввода в правом сайдбаре получаем значение - метку вершины
      const vertexLabel = document.getElementById(
        "right-sidebar__input__id"
      ).value;

      // Очищаем поле ввода
      document.getElementById("right-sidebar__input__id").value = "";

      // Проверяем корректность введенной метки вершины
      if (this.#isVertexLabelValid(vertexLabel)) {
        // ID пути (path) для построения текста метки по нему
        const pathID = "vertex" + this.vertexID + "_path";
        // ID метки (label) вершины
        const labelID = "vertex" + this.vertexID + "_label";

        // Построения текста по пути (path)
        CanvasCreateTextByPath(
          path,
          vertexLabel,
          pathID,
          labelID,
          50,
          this.radius / 10
        );

        // Разблокируем действия на странице
        actionsDisabled = false;
        // Скрываем поля ввода в правом сайдбаре
        RightSidebarBlock.style.display = "none";
        // Добавляем метку вершину в коллекцию меток вершин
        this.createdVerticesLabels.add(vertexLabel);
        // Для кнопки подтверждения в правом сайдбаре удаляем обработчик на клик
        RightSidebarConfirmButton.removeEventListener("click", setLabelHandler);

        // Заносим всю информацию в необходимые переменные-члены графа
        this.vertices[id] = {};
        this.vertices[id]["edges"] = [];
        this.vertices[id]["metadata"] = {
          position: {
            // позиция построенной вершины
            x: positionX,
            y: positionY,
          },
          label: vertexLabel, // метка построенной вершины
          label_metadata: {
            // метаданные построенной вершины (ID пути, ID метки - необходимо для
            // последующей выборки этих элементов в графе)
            pathID: pathID,
            labelID: labelID,
          },
        };

        CanvasChangeBorderColor([id], "#000000");
        DOMChangeColor(lastButtonSelected);
        setInformationConsoleMessage(
          `Вы успешно добавили метку для вершины. Выберите действие`
        );
      }
    };

    // На кнопку подтверждения в правом сайдбаре вешаем обработчик на клик
    RightSidebarConfirmButton.addEventListener("click", setLabelHandler);

    // Успешно создали вершину -> return true
    return true;
  }

  /**
   * Публичная функция-член класса, реализующая бизнес-логику по удалению вершины
   *
   * @param vertexID - id удаляемой вершины
   */
  DeleteVertex(vertexID) {
    // Удаляем с холста все элементы связанные непосредственно с вершиной (path по которому строился
    // текст метки вершины, метка вершины), кроме ребер
    Object.values(
      this.vertices[vertexID]["metadata"]["label_metadata"]
    ).forEach((id) => {
      // На холсте находим элемент по его ID и удаляем его
      svgCanvas.getElementById(id).remove();
    });

    // Удаляем саму вершину с холста
    svgCanvas.getElementById(vertexID).remove();

    // Удаляем ребра, с которыми связана вершина, для этого в цикле forEach проходим по массиву ребер для
    // удаляемой вершины
    this.vertices[vertexID]["edges"].forEach((edge) => {
      // Для каждого ребра необходимо удалить все элементы на холсте с которыми непосредственно связанно ребро,
      // кроме других вершин в которое входило или из которых выходило ребро (линия соединяющая вершины, path
      // по которому строился текст метки ребра, метка ребра)
      Object.values(edge["metadata"]).forEach((id) => {
        // На холсте находим элемент по его ID и удаляем его
        svgCanvas.getElementById(id).remove();
      });

      // Для текущего ребра по ключу 'value' получаем ID вершины в которою входило или из которой выходило ребро
      const connectedVertexID = edge["value"];

      // Удаляем всю информацию о ребре из связанной с ним вершины. Мы удаляем ребро с холста => должны удалить
      // всю информацию об этом ребре из другой - связанной с ним вершины
      this.#deleteEdgeFromConnectedVertices(vertexID, connectedVertexID, edge);

      // Проверяем свойство 'parallelism' у связанной вершины
      this.#checkParallelismProperty(connectedVertexID);
    });

    // Удаляем метку вершины из коллекции меток, таким образом при создании новой вершины можно будет переиспользовать
    // метку удаленной ранее вершины
    this.createdVerticesLabels.delete(
      this.vertices[vertexID]["metadata"]["label"]
    );

    // Удаляем вершину из массива уже построенных вершин, таким образом при построении новой вершины положение
    // уже удаленной вершины не должно и не будет учитываться
    this.createdVerticesPositions = this.createdVerticesPositions.filter(
      (vertex) => vertex[0] !== vertexID
    );

    // Из объекта vertices, который хранит все вершины графа удаляем вершину
    delete this.vertices[vertexID];

    DOMChangeColor(lastButtonSelected);
    setInformationConsoleMessage("Вершина успешно удалена");
  }

  /**
   * Публичная функция-член класса, реализующая бизнес-логику по удалению ребра
   *
   * @param edgeID - ID удаляемого ребра
   */
  DeleteEdge(edgeID) {
    // Объявления переменные для хранения ID вершин между которыми построено ребро
    let fistVertexID, secondVertexID;

    // В цикле forEach итерируемся по ключам объекта vertices. Наша цель - найти две вершины между которыми построено
    // ребро. Заметим, что мы бы могли здесь использовать Object.values(), но нам нужен именно ключ объекта vertices,
    // поскольку ключ - это ID вершины, а для корректного удаления ребра нам необходимо иметь ID вершин.
    Object.keys(this.vertices).forEach((vertex) => {
      // Для каждой вершины в цикле forEach итерируемся по массиву ребер
      this.vertices[vertex]["edges"].forEach((edge) => {
        // Если нашли нужное нам ребро
        if (edge["metadata"]["edgeID"] === edgeID) {
          // Для каждого ребра необходимо удалить все элементы на холсте с которыми непосредственно связанно ребро,
          // кроме других вершин в которое входило или из которых выходило ребро (линия соединяющая вершины, path
          // по которому строился текст метки ребра, метка ребра)
          Object.values(edge["metadata"]).forEach((id) => {
            // На холсте находим элемент по его ID и удаляем его
            svgCanvas.getElementById(id).remove();
          });

          fistVertexID = vertex; // ID первой вершины
          secondVertexID = edge["value"]; // ID второй вершины, в по ключу "value" в объекте edge хранится ID
          // сопряженной вершины

          // Удаляем всю информацию о ребре из массива ребер для каждой вершины
          this.#deleteEdgeFromConnectedVertices(
            fistVertexID,
            secondVertexID,
            edge
          );

          // Проверяем свойство 'parallelism' у первой вершины
          this.#checkParallelismProperty(fistVertexID);
          // Проверяем свойство 'parallelism' у второй  вершины
          this.#checkParallelismProperty(secondVertexID);

          DOMChangeColor(lastButtonSelected);
          setInformationConsoleMessage("Вы успешно удалили ребро");
        }
      });
    });
  }

  /**
   * Публичная функция-член, реализующая бизнес-логику по добавлению ребра
   *
   * @param fromVertexID - ID вершины из которой выходит ребро
   * @param toVertexID - ID вершины в которую приходит ребро
   * @returns {boolean} - true - успешно построили ребро, false - произошла ошибка
   */
  AddEdge(fromVertexID, toVertexID) {
    CanvasChangeBorderColor([fromVertexID, toVertexID], "#000000");
    // Проверка, что пользователь не пытается построить ребро из вершины в нее же саму
    if (fromVertexID === toVertexID) {
      setInformationConsoleMessage(
        `Вы не можете построить ребро из вершины в нее саму. Попробуйте снова, выберите первую вершину`
      );
      return false;
    }

    // Получаем количество ребер выходящих из вершины fromVertex и приходящих в вершину toVertex
    const amount = this.#countEdgesAmount(fromVertexID, toVertexID);

    // Получаем позицию вершины fromVertex по координате X и координате Y
    const fromVertexPositionX =
      this.vertices[fromVertexID]["metadata"]["position"]["x"];
    const fromVertexPositionY =
      this.vertices[fromVertexID]["metadata"]["position"]["y"];
    // Получаем позицию вершины toVertex по координате X и координате Y
    const toVertexPositionX =
      this.vertices[toVertexID]["metadata"]["position"]["x"];
    const toVertexPositionY =
      this.vertices[toVertexID]["metadata"]["position"]["y"];

    // ID ребра
    const id = "edge" + ++this.edgeID;

    let edgeType, path;
    if (amount === 0) {
      // Если отсутствуют ребра исходящие из вершины fromVertex и приходящие в вершину toVertex, то
      // строим первое ребро с помощью обычной прямой линии
      path = CanvasCreateStraightEdge(
        fromVertexPositionX,
        fromVertexPositionY,
        toVertexPositionX,
        toVertexPositionY,
        this.radius,
        id,
        this.arrowheadSize
      );
      edgeType = "straight"; // Тип построения ребра - прямое
    } else {
      // Если есть ребра исходящие из вершины fromVertex и приходящие в вершину toVertex, то
      // строим ребро с использованием кривых Безье
      path = CanvasCreateBezierEdge(
        fromVertexPositionX,
        fromVertexPositionY,
        toVertexPositionX,
        toVertexPositionY,
        this.radius,
        id,
        this.arrowheadSize,
        amount * 3
      );
      edgeType = "bezier"; // Тип построения ребра - кривая Безье
    }
    CanvasChangeBorderColor([id], "#B92808");

    // true - из вершины выходит более 1 ребра, следовательно необходимо уточнение параллелизма, если этого свойства
    // не было ранее
    const needParallelismSelection =
      this.#checkParallelismForVertex(fromVertexID);
    // Рендеринг полей ввода для предиката и функции ребра, если needParallelismSelection=true, то также ренедериться
    // селектор выбора типа параллелизма
    renderEdgeInputFields(needParallelismSelection);
    // Включаем видимость правого сайдбара
    RightSidebarBlock.style.display = "flex";

    // Информационное сообщение в консоль (нижний сайдбар)
    setInformationConsoleMessage(
      `Вы успешно добавили ребро. Теперь введите предикат и функцию для ребра в полях ввода на правой панели`
    );

    // Блокируем действия на странице пока не будет завершено построение ребра и уточнение всех необходимых свойств
    actionsDisabled = true;

    // Необходимо ли уточнение информации о предикате или функции?
    let needPredicateMeta = false,
      needFunctionMeta = false;
    // Пустые переменные в которых будет храниться введенный предикат и функция после их считывания из полей ввода
    let predicate = "",
      func = "";

    /**
     * Функция-обработчик клика на кнопку подтверждения в правом сайдбаре
     */
    const setLabelHandler = () => {
      // Если нет необходимости в уточнении информации о предикате и функции (заметим, что изначально переменные
      // инициализированы значением false)
      if (!needPredicateMeta && !needFunctionMeta) {
        // Объект тега <input> для ввода информации о предикате
        const inputPredicate = document.getElementById(
          "right-sidebar__input-predicate__id"
        );
        // Объект тега <input> для ввода информации о функции
        const inputFunction = document.getElementById(
          "right-sidebar__input-function__id"
        );

        // Получаем значения из полей ввода
        predicate = inputPredicate.value;
        func = inputFunction.value;

        // Объект тега <select> для выбора типа параллелизма
        const selector = document.getElementById("selectorParallelism_id");
        // Если тег <select> был на странице (!== null), то получаем значение из него и сразу сохраняем его как
        // свойство вершины fromVertex
        if (selector !== null) {
          this.vertices[fromVertexID]["parallelism"] =
            selector[selector.selectedIndex].text;
        }

        // Согласно формату aDOT мы можем опустить ввод предиката, но значение функции должно быть введено, поэтому
        // проверяем, что значение из поля ввода функции не пустое
        if (func.length !== 0) {
          // ID пути по которому будет строиться ребро
          const pathID = "edge" + this.edgeID + "_path";
          // ID метки для ребра
          const labelID = "edge" + this.edgeID + "_label";
          // Текст метки для ребра
          const label =
            predicate.length === 0
              ? "<" + func + ">"
              : "<" + predicate + ", " + func + ">";
          // Строим текст метки по полученному ранее пути
          CanvasCreateTextByPath(path, label, pathID, labelID, 50, -10);

          // Получаем флаги: уникален ли предикат или функция в рамках графа
          [needPredicateMeta, needFunctionMeta] =
            this.#isNewPredicateAndFunction(predicate, func);

          // Если предикат и функция не уникальны в рамках графа
          if (!needFunctionMeta && !needPredicateMeta) {
            // Завершаем построение ребра, у кнопки подтверждения ввода на правом сайдбаре удаляем обработчик на клик
            RightSidebarConfirmButton.removeEventListener(
              "click",
              setLabelHandler
            );
            // Разблокируем действия на странице
            actionsDisabled = false;
            // Скрываем видимость блока правого сайдбара
            RightSidebarBlock.style.display = "none";

            // Показываем пользователю информационное сообщение в консоли (нижний сайдбар)
            DOMChangeColor(lastButtonSelected);
            CanvasChangeBorderColor([id], "#000000");
            setInformationConsoleMessage(
              `Вы успешно добавили метку для ребра.`
            );
          } else {
            // Если необходимо уточнение информации о предикате
            if (needPredicateMeta && !needFunctionMeta) {
              // Показываем пользователю информационное сообщение в консоли (нижний сайдбар)
              setInformationConsoleMessage(`Вы успешно добавили метку для ребра. Теперь введите метаинформацию о новом предикате в поле
                            ввода на правой панели`);

              // Если необходимо уточнение информации о функции
            } else if (!needPredicateMeta && needFunctionMeta) {
              // Показываем пользователю информационное сообщение в консоли (нижний сайдбар)
              setInformationConsoleMessage(`Вы успешно добавили метку для ребра. Теперь введите метаинформацию о новой функции в поле
                            ввода на правой панели`);

              // Если необходимо уточнение информации и о предикате и о функции
            } else {
              // Показываем пользователю информационное сообщение в консоли (нижний сайдбар)
              setInformationConsoleMessage(`Вы успешно добавили метку для ребра. Теперь введите метаинформацию о новом предикате и функции
                            в полях ввода на правой панели`);
            }

            // Рендерим поля ввода для уточнения информации о предикате и функции
            renderEdgeInputFieldPredicateFunction(
              needPredicateMeta,
              needFunctionMeta
            );
          }

          // В массив ребер для вершины fromVertex добавляем новый объект, который содержит всю информацию о созданном ребре
          this.vertices[fromVertexID]["edges"].push(
            // Для первой вершины
            {
              value: toVertexID,
              type: edgeType,
              direction: "from",
              label: label,
              predicate: predicate,
              function: func,
              metadata: {
                edgeID: id,
                pathID: pathID,
                labelID: labelID,
              },
            }
          );

          // В массив ребер для вершины toVertex добавляем новый объект, который содержит всю информацию о созданном ребре
          this.vertices[toVertexID]["edges"].push({
            value: fromVertexID,
            direction: "to",
            type: edgeType,
            label: label,
            predicate: predicate,
            function: func,
            metadata: {
              edgeID: id,
              pathID: pathID,
              labelID: labelID,
            },
          });

          // Если необходимо уточнение информации о предикате, то заранее в объекте predicates по ключу predicate
          // создаем пустой объект
          if (needPredicateMeta) {
            this.predicates[predicate] = {};
          }

          // Если необходимо уточнение информации о функции, то заранее в объекте functions по ключу func
          // создаем пустой объект
          if (needFunctionMeta) {
            this.functions[func] = {};
          }

          // Функция не может быть пустой
        } else {
          // Показываем пользователю информационное сообщение в консоли (нижний сайдбар)
          setInformationConsoleMessage(
            `Поле для ввода функции не может быть пустым`
          );
        }

        // Если необходимо уточнение информации о предикате или функции
      } else {
        // Объект тега <input> для ввода свойства module для предиката
        const predicateInputModule = document.getElementById(
          "right-sidebar__input-predicate-module__id"
        );
        // Объект тега <input> для ввода свойства entry func для предиката
        const predicateInputEntryFunc = document.getElementById(
          "right-sidebar__input-predicate-entry__id"
        );

        // Объект тега <input> для ввода свойства module для функции
        const functionInputModule = document.getElementById(
          "right-sidebar__input-function-module__id"
        );
        // Объект тега <input> для ввода свойства entry func для функции
        const functionInputEntryFunc = document.getElementById(
          "right-sidebar__input-function-entry__id"
        );

        // Пустые переменные, в которые далее будут сохраняться соответствующие свойства предиката и функции
        let predicateModule = "",
          predicateEntryFunc = "",
          functionModule = "",
          functionEntryFunc = "";

        // Если необходимо уточнение свойства предиката, то получаем значения module и entry func и соответствующих
        // полей ввода
        if (needPredicateMeta) {
          predicateModule = predicateInputModule.value;
          predicateEntryFunc = predicateInputEntryFunc.value;
        }
        // Если необходимо уточнение свойства функции, то получаем значения module и entry func и соответствующих
        // полей ввода
        if (needFunctionMeta) {
          functionModule = functionInputModule.value;
          functionEntryFunc = functionInputEntryFunc.value;
        }

        // Флаг для проверки, что поля заполнены
        let flag = true;

        // Если необходимо уточнение свойств предиката, но поля ввода свойств не заполнены
        if (
          needPredicateMeta &&
          (predicateModule.length === 0 || predicateEntryFunc.length === 0)
        ) {
          // Показываем пользователю информационное сообщение в консоли (нижний сайдбар)
          setInformationConsoleMessage(`Поля должны быть заполнены`);
          flag = false; // поля не заполнены
        }

        // Если необходимо уточнение свойств функции, но поля ввода свойств не заполнены
        if (
          needFunctionMeta &&
          (functionModule.length === 0 || functionEntryFunc.length === 0)
        ) {
          // Показываем пользователю информационное сообщение в консоли (нижний сайдбар)
          setInformationConsoleMessage(`Поля должны быть заполнены`);
          flag = false; // поля не заполнены
        }

        // Если flag=true - поля успешно заполнены
        if (flag) {
          // Если было необходимо уточнение свойств предиката, то сохраняем эти свойства по соответствующим ключам
          // в объект predicates
          if (needPredicateMeta) {
            this.predicates[predicate]["module"] = predicateModule;
            this.predicates[predicate]["entry_func"] = predicateEntryFunc;
          }
          // Если было необходимо уточнение свойств функции, то сохраняем эти свойства по соответствующим ключам
          // в объект functions
          if (needFunctionMeta) {
            this.functions[func]["module"] = functionModule;
            this.functions[func]["entry_func"] = functionEntryFunc;
          }

          // Завершаем построение ребра, у кнопки подтверждения ввода на правом сайдбаре удаляем обработчик на клик
          RightSidebarConfirmButton.removeEventListener(
            "click",
            setLabelHandler
          );
          // Разблокируем действия на странице
          actionsDisabled = false;
          // Скрываем видимость блока правого сайдбара
          RightSidebarBlock.style.display = "none";
          // Показываем пользователю информационное сообщение в консоли (нижний сайдбар)

          DOMChangeColor(lastButtonSelected);
          CanvasChangeBorderColor([id], "#000000");
          setInformationConsoleMessage(
            `Вы успешно добавили метаинформацию для ребра. Выберите действие`
          );
        }
      }
    };

    // Вешаем обработчик на клик на кнопку подтверждения ввода на правом сайдбаре
    RightSidebarConfirmButton.addEventListener("click", setLabelHandler);

    // Построение ребра успешно завершено
    return true;
  }

  /**
   * Публичная функция-член, реализующая бизнес-логику по экспорту графа в формат aDOT
   *
   * @param startVertexID - ID стартовой вершины графа
   * @param endVertexID - ID конечной вершины графа
   * @returns {boolean} - true - экспорт завершен успешно
   */
  ExportADOT(startVertexID, endVertexID) {
    CanvasChangeBorderColor([startVertexID, endVertexID], "#000000");
    // Получаем текстовое описание графа в формате aDOT
    const text = this.#parseToADOT(startVertexID, endVertexID);

    // Если успешно получили текстовое описание, то скачиваем его как файл в формате .txt
    if (text) {
      const filename = "somefile.txt";
      const file = new Blob([text], { type: "text/plain" });
      const a = document.createElement("a"),
        url = URL.createObjectURL(file);
      a.href = url;
      a.download = filename;
      document.body.appendChild(a);
      a.click();
      setTimeout(() => {
        document.body.removeChild(a);
        window.URL.revokeObjectURL(url);
      }, 0);

      DOMChangeColor(lastButtonSelected);
      // Экспорт завершен успешно
      return true;
    }

    // Во время экспорта произошла ошибка
    return false;
  }

  /**
   * Функция реализующая бизнес-логику по визуализации графа из формата aDOT
   *
   * @param data - текст файла в формате aDOT
   */
  ImportADOT(data) {
    // Очищаем холст
    this.Clear();

    const lines = data.split("\n"); // Разбиваем файл на массив строк

    let idx = this.#parseMetadata(lines); // Парсинг метаинформации (параллелизм у вершин, предикаты, функции, ребра (edges))
    // В качестве значения функция возвращает индекс в массиве lines на котором заканчивается метаинформация и начинается
    // описание графовой модели

    this.#renderVertices(idx, lines); // Визуализация вершин графа (самый интересный алгоритм в проекте=) )
    this.#renderEdges(idx, lines);
    this.#parseParallelism(lines);

    setInformationConsoleMessage(
      "Граф успешно импортирован. Выберите действие"
    );
  }

  /**
   * Публичная функция-член реализующая полную очистку экрана
   */
  Clear = () => {
    this.vertices = {}; // Очищаем объект vertices
    this.vertexID = 0; // ID вершин будут начинаться с 1
    this.edgeID = 0; // ID ребер будет начинаться с 1
    this.createdVerticesPositions = []; // Очищаем массив содержащий информацию о созданных вершинах
    this.createdVerticesLabels = new Set(); // Пересоздаем коллекцию меток вершин
    this.predicates = {}; // Очищаем объект predicates
    this.functions = {}; // Очищаем объект functions
    this.edges = {}; // Очищаем объект edges
    CanvasClearAll(); // Очищаем сам холст
    setInformationConsoleMessage("Холст успешно очищен");
  };

  FindCycles() {}

  /**
   * Приватная функция-член класс, которая проверяет позицию вершины. Для этого мы итерируемся по массиву
   * createdVerticesPositions, и для текущего элемента массива (уже созданной вершины) получаем: положение вершины по
   * X, положение вершины по Y, радиус вершины. Заметим, что мы хотим ограничить не только пересечение вершин, а также
   * создание вершины в непосредственной близости от другой. Для этого мы умножаем радиус вершины на 2, тем самым
   * создавая мнимую область вокруг вершины, в которой не могут располагаться другие вершины.
   *
   * @param x - позиция вершины по X
   * @param y - позиция вершины по Y
   * @returns {boolean} - true - не пересекается, false - пересекается
   */
  #isVertexPositionValid = (x, y) => {
    for (const current of this.createdVerticesPositions) {
      const [currentX, currentY] = current.slice(1);
      const d = Math.sqrt((currentX - x) ** 2 + (currentY - y) ** 2);
      if (d < 2 * this.radius * 2) {
        return false;
      }
    }
    return true;
  };

  /**
   * Приватная функция-член класса, которая проверяет корректность метки вершины. В функции есть несколько проверок:
   *      1) Проверка на корректность символов, в приложении для меток доступны только латинские буквы (маленькие и
   *      большие), а также цифры.
   *      2) Проверка на длину текста, пользователь не может задать пустую строку в качестве метки, а также слишком
   *      длинную строку (по умолчанию больше 5 символов)
   *      3) Проверка, что метка вершины уникальна в рамках графа
   *
   * @param label - текст
   * @returns {boolean} - true - метка валидна, false - метка невалидна
   */
  #isVertexLabelValid = (label) => {
    // 1) Проверка на корректность символов. Итерируемся по каждому символу строки
    for (let i = 0; i < label.length; ++i) {
      // Получаем ASCII код символа
      const character = label.charCodeAt(i);
      // Проверяем код символа по таблице ASCII (http://book.itep.ru/10/ascii.htm)
      if (
        (character < 97 && character > 90) ||
        character > 122 ||
        (character < 65 && character > 57) ||
        character < 48
      ) {
        setInformationConsoleMessage(
          `Некорректная метка для вершины. Доступны маленькие/большие латинские буквы и цифры`
        );
        // Метка невалидна -> return false
        return false;
      }
    }
    // 2) Проверка на корректность длины
    if (label.length === 0 || label.length > maxLabelLength) {
      setInformationConsoleMessage(`Метка для вершины не может быть пустой`);
      // Метка невалидна -> return false
      return false;
    }
    // 3) Проверка на уникальность метки в рамках графа. createdVerticesLabels - коллекция (set), с помощью метода
    // has проверяем что в нем присутствует/отсутствует метка
    if (this.createdVerticesLabels.has(label)) {
      setInformationConsoleMessage(
        `Вершина с такой меткой уже существует. Попробуйте еще раз`
      );
      return false;
    }
    return true;
  };

  /**
   * Приватная функция-члена класса, которая удаляет информацию о ребре у связанных с ним вершин
   *
   * @param firstVertexID - ID первой вершины
   * @param secondVertexID - ID второй вершины
   * @param edge - Удаляемое ребро
   */
  #deleteEdgeFromConnectedVertices = (firstVertexID, secondVertexID, edge) => {
    console.log("Delete:", firstVertexID, secondVertexID, edge);
    // Если предикат используется только для переданного ребра, то удаляем информацию о нем из объекта predicates
    if (this.#isLastUsedEdgePredicate(edge["predicate"])) {
      delete this.predicates[edge["predicate"]];
    }
    // Если функция используется только для переданного ребра, то удаляем информацию о ней из объекта functions
    if (this.#isLastUsedEdgeFunction(edge["function"])) {
      delete this.functions[edge["function"]];
    }

    // Удаляем ребро из массива ребер первой вершины
    this.vertices[firstVertexID]["edges"] = this.vertices[firstVertexID][
      "edges"
    ].filter((e) => e["metadata"]["edgeID"] !== edge["metadata"]["edgeID"]);
    // Удаляем ребро из массива ребер второй вершины
    this.vertices[secondVertexID]["edges"] = this.vertices[secondVertexID][
      "edges"
    ].filter((e) => e["metadata"]["edgeID"] !== edge["metadata"]["edgeID"]);
  };

  /**
   * Приватная функция-член класса, проверяющая, что переданный ей предикат используется только для одного ребра
   *
   * @param predicate - предикат
   * @returns {boolean} - true - используется только для одного ребра
   */
  #isLastUsedEdgePredicate = (predicate) => {
    let amount = 0;
    Object.values(this.vertices).forEach((vertex) => {
      amount += vertex["edges"].filter(
        (edge) => edge["predicate"] === predicate
      ).length;
    });
    return amount === 2;
  };

  /**
   * Приватная функция-член класса, проверяющая, что переданная ей функция используется только для одного ребра
   *
   * @param func - функция
   * @returns {boolean} - true - используется только для одного ребра
   */
  #isLastUsedEdgeFunction = (func) => {
    let amount = 0;
    Object.values(this.vertices).forEach((vertex) => {
      amount += vertex["edges"].filter(
        (edge) => edge["function"] === func
      ).length;
    });
    return amount === 2;
  };

  /**
   * Приватная функция-член, проверяющая корректность наличия свойства 'parallelism' у переданной вершины.
   * Если у вершины было свойство 'parallelism' => из нее выходил более 1 ребра, нам необходимо
   * пересчитать количество ребер, которые выходят из данной вершины, поскольку в процессе удаления ребер
   * удаляемой вершины мы могли удалить ребро, которое выходит из связанной вершины. Следовательно мы могли
   * нарушить условие наличия у вершины свойства 'parallelism', поэтому мы проверяем, что у связанной вершины
   * было свойство 'parallelism' и что новое количество выходящих ребер равно 1, в таком случае мы удаляем
   * у вершины свойство 'parallelism'
   *
   * @param vertexID - ID вершины
   */
  #checkParallelismProperty = (vertexID) => {
    if (
      this.vertices[vertexID].hasOwnProperty("parallelism") &&
      this.vertices[vertexID]["edges"].filter(
        (edge) => edge["direction"] === "from"
      ).length === 1
    ) {
      delete this.vertices[vertexID]["parallelism"];
    }
  };

  /**
   * Приватная функция-член, проверяющая что из вершины выходит более одного ребра => необходимо показать пользователю
   * селектор выбора типа параллелизма
   *
   * @param vertexID - ID вершины
   * @returns {boolean} - true - необходимо уточнение типа параллелизма
   */
  #checkParallelismForVertex = (vertexID) => {
    // Если для вершины уже выбрал тип параллелизма
    if (this.vertices[vertexID].hasOwnProperty("parallelism")) {
      return false;
    }

    // Если количество выходящих ребер >= 1 - true, иначе false
    return (
      this.vertices[vertexID]["edges"].filter(
        (edge) => edge["direction"] === "from"
      ).length >= 1
    );
  };

  /**
   * Приватная функция-член, вычисляющая количество ребер, из вершины fromVertex в вершину toVertex
   *
   * @param fromVertexID - ID вершины из которой выходит ребро
   * @param toVertexID - ID вершины в которую приходит ребро
   * @returns {number} - количество ребер между вершинами
   */
  #countEdgesAmount = (fromVertexID, toVertexID) => {
    // Получаем количество уже построенных ребер из вершины fromVertex в вершину toVertex
    let amount = this.vertices[fromVertexID]["edges"].filter(
      (edge) => edge["value"] === toVertexID && edge["direction"] === "from"
    ).length;

    // Если существует ребро из вершины toVertex в вершину fromVertex и у ребра по ключу "type" находится свойство
    // "straight" => ребро было построено прямой линией, значит мы должны представить, что у нас уже есть прямое ребро
    // из вершины fromVertex в вершину toVertex => мы инкрементируем счетчик ребер и это ребро будет строиться с
    // использованием кривых Безье. Иначе просто возвращаем количество ребер.
    return this.vertices[toVertexID]["edges"].filter(
      (edge) =>
        edge["value"] === fromVertexID &&
        edge["direction"] === "from" &&
        edge["type"] === "straight"
    ).length > 0
      ? amount + 1
      : amount;
  };

  /**
   * Приватная функция-член, проверяющая что введенный предикат или функции - уникальны в рамках графика => для них
   * требуется уточнение свойств (entry module, entry func)
   *
   * @param predicate - предикат
   * @param func - функция
   * @returns {[boolean, boolean]} - 1-й элемент результат для предиката, 2-й элемент результат для функции
   */
  #isNewPredicateAndFunction = (predicate, func) => {
    // Начальная инициализация флагов
    let isNewPredicate = true,
      isNewFunc = true;

    // Если в объекте predicates уже есть ключ равный переданному predicate или переданный predicate пустой, то
    // предикат не уникален в рамках графа => false
    if (predicate in this.predicates || predicate.length === 0) {
      isNewPredicate = false;
    }

    // Если в объекте functions уже есть ключ равный переданному func, то функция не уникальна в рамках графа => false
    if (func in this.functions) {
      isNewFunc = false;
    }

    return [isNewPredicate, isNewFunc];
  };

  /**
   * Приватная функция-член, выполняющая парсинг графа в формат aDOT
   *
   * @param startVertexID - ID стартовой вершины
   * @param endVertexID - ID конечной вершины
   * @returns {string|undefined} - текстовое описание графа в aDOT или undefined если произошла ошибка
   */
  #parseToADOT = (startVertexID, endVertexID) => {
    // Проверка, что стартовая и конечная вершины различаются
    if (startVertexID === endVertexID) {
      // Показываем пользователю информационное сообщение в консоли (нижний сайдбар)
      setInformationConsoleMessage(
        "Стартовая и конечная вершины должны быть различными. Попробуйте еще раз, выберите стартовую" +
          "вершину кликнув на нее или выберите другое действие"
      );

      // Произошла ошибка => не может получить текстовое представление графа в формате aDOT
      return undefined;
    }

    // Из стартовой вершины выходит хотя бы одно ребро?
    const isStartVertexValid = this.#checkDirectionForEdges(
      startVertexID,
      "from"
    );
    // В конечную вершину приходит хотя бы одно ребро?
    const isEndVertexValid = this.#checkDirectionForEdges(endVertexID, "to");

    // Если из стартовой вершины не выходит ни одного ребра
    if (!isStartVertexValid) {
      // Показываем пользователю информационное сообщение в консоли (нижний сайдбар)
      setInformationConsoleMessage(
        "Стартовая вершина выбрана некорректно, из стартовой вершины должно выходить как минимум одно ребро. " +
          "Попробуйте еще раз, выберите стартовую вершину кликнув на нее или выберите другое действие"
      );

      // Произошла ошибка => не может получить текстовое представление графа в формате aDOT
      return undefined;
    }
    // Если в стартовую вершину не приходит ни одного ребра
    if (!isEndVertexValid) {
      // Показываем пользователю информационное сообщение в консоли (нижний сайдбар)
      setInformationConsoleMessage(
        "Конечная вершина выбрана некорректно, в конечную вершину должно входить как минимум одно ребро. " +
          "Попробуйте еще раз, выберите стартовую вершину кликнув на нее или выберите другое действие"
      );

      // Произошла ошибка => не может получить текстовое представление графа в формате aDOT
      return undefined;
    }

    //------Создаем объект edges, каждое ребро - это предикат и функция, которые используются для ребер на графе
    // Начинаем с очистки объекта edges
    this.edges = {};

    // Переменная, которая хранит индекс edge
    let edgeIndex = 0;

    // Итерируемся по всем вершинам объекта vertices
    Object.values(this.vertices).forEach((vertex) => {
      // Для каждой вершины итерируемся по массиву ее ребер
      vertex["edges"].forEach((current) => {
        let isUniqueCombination = true, // флаг для хранения информации о том что комбинация predicate и function уникальна
          id = ""; // id ребра edge

        // Итерируемся по объекту edges чтобы проверить уникальность текущей комбинации predicate и function
        Object.keys(this.edges).forEach((edge) => {
          // Если уже есть edge с такой комбинацией predicate и function
          if (
            this.edges[edge]["predicate"] === current["predicate"] &&
            this.edges[edge]["function"] === current["function"]
          ) {
            isUniqueCombination = false; // меняем значение флага isUniqueCombination на false
            id = edge; // ID найденного edge
          }
        });

        // Если это уникальная комбинация predicate и function
        if (isUniqueCombination) {
          // У объекта edges по ключу edge_<edgeIndex> создаем пустой объект
          this.edges["edge_" + ++edgeIndex] = {};
          // В созданный объект добавляем ключи predicate и function с соответствующими значениями
          this.edges["edge_" + edgeIndex]["predicate"] = current["predicate"];
          this.edges["edge_" + edgeIndex]["function"] = current["function"];

          // Для текущего просматриваемого ребра добавляем ключ "edge" по которому записываем edge к которому относится
          // это ребро
          current["edge"] = "edge_" + edgeIndex;
        } else {
          // Иначе добавляем ключ "edge" по которому записываем edge к которому относится
          // это ребро
          current["edge"] = id;
        }
      });
    });

    // Создаем переменную которая будет хранить текстовое описание графа
    let data = "digraph TEST\n{\n";

    //------Парсинг
    // 1) Параллелизм
    let hasParallelism = false; // флаг, необходим для хранения информации о том что в графе присутствует хотя бы
    // одна вершина со свойством "parallelism"

    // Проходим по всем вершинам и проверяем есть ли у вершины свойство "parallelism"
    Object.values(this.vertices).forEach((current) => {
      if (current.hasOwnProperty("parallelism")) {
        hasParallelism = true; // если найдена вершина, то флаг - true
      }
    });

    // Если в графе есть хотя бы одна вершина со свойством "parallelism"
    if (hasParallelism) {
      data += "// В узле указана стратегия распараллеливания\n";
      // Каждую вершину для которой есть свойство "parallelism" необходимо описать в текстовом файле, для этого
      // итерируемся по всем вершинам, проверяем есть ли свойство "parallelism" и если оно есть, то в переменную
      // data записываем соответствующую вершину c указанием типа параллелизма
      Object.values(this.vertices).forEach((currentVertex) => {
        if (currentVertex.hasOwnProperty("parallelism")) {
          if (currentVertex["parallelism"].length) {
            data =
              data +
              `\t${currentVertex["metadata"]["label"]} [parallelism=${currentVertex["parallelism"]}]\n`;
          }
        }
      });
    }

    // 2) Функции-обработчики
    data += "// Определения функций−обработчиков\n";
    // Итерируемся по объекту functions и в переменную data заносим всю необходимую информацию
    Object.keys(this.functions).forEach((currentFunction) => {
      data =
        data +
        `\t${[currentFunction]} [module=${
          this.functions[currentFunction]["module"]
        }, entry_func=${this.functions[currentFunction]["entry_func"]}]\n`;
    });

    // 3) Функции-предикаты
    data += "// Определения функций−предикатов\n";
    // Итерируемся по объекту predicates и в переменную data заносим всю необходимую информацию
    Object.keys(this.predicates).forEach((currentPredicate) => {
      data =
        data +
        `\t${[currentPredicate]} [module=${
          this.predicates[currentPredicate]["module"]
        }, entry_func=${this.predicates[currentPredicate]["entry_func"]}]\n`;
    });

    // 4) Функции-перехода (edges)
    data += "// Определения функций перехода\n";
    // Итерируемся по объекту edges и в переменную data заносим всю необходимую информацию
    Object.keys(this.edges).forEach((currentEdge) => {
      data =
        data +
        `\t${[currentEdge]} [predicate=${
          this.edges[currentEdge]["predicate"]
        }, function=${this.edges[currentEdge]["function"]}]\n`;
    });

    // 5) Описание графовой модели
    data += "// Описание графовой модели\n";
    // Стартовая вершина
    data =
      data +
      `\t__BEGIN__ -> ${this.vertices[startVertexID]["metadata"]["label"]}\n`;

    // Итерируемся по объекту vertices
    Object.values(this.vertices).forEach((vertex) => {
      // Стрелка в текстовом формате, которая показывает, что между вершинами есть ребро. Заметим, что по умолчанию
      // стрелка "->", но если у вершины есть свойство "parallelism" со значением "threading", то стрелка меняется на "=>"
      const transition =
        vertex.hasOwnProperty("parallelism") &&
        vertex["parallelism"] === "threading"
          ? "=>"
          : "->";

      // Итерируемся по массиву ребер для каждой вершины
      vertex["edges"].forEach((currentEdge) => {
        // Если ребро выходит из вершины, то записываем его в переменную data в следующем формате
        // "<вершина откуда выходит ребро> <стрелка перехода> <вершина куда приходит ребро> [morphism=<edge>]"
        if (currentEdge["direction"] === "from") {
          data =
            data +
            `\t${vertex["metadata"]["label"]} ${transition} ${
              this.vertices[currentEdge["value"]]["metadata"]["label"]
            } [morphism=${currentEdge["edge"]}]\n`;
        }
      });
    });

    // Конечная вершина
    data =
      data +
      `\t${this.vertices[endVertexID]["metadata"]["label"]} -> __END__ \n`;
    data += "}\n";

    return data;
  };

  /**
   * Приватная функция-член, реализующая проверку, что у вершины есть хотя бы одно ребро с заданным направлением
   *
   * @param vertexID - ID вершины
   * @param direction - направление
   * @returns {boolean} - true - у вершины есть ребро с заданным направлением
   */
  #checkDirectionForEdges = (vertexID, direction) => {
    return (
      this.vertices[vertexID]["edges"].filter(
        (edge) => edge["direction"] === direction
      ).length > 0
    );
  };

  /**
   * Приватная функция-член, реализующая парсинг метаинформации о графе
   *
   * @param lines - массив строк переданного файла в формате aDOT
   * @returns {number} - индекс в массиве, на котором завершился парсинг метаинформации
   */
  #parseMetadata = (lines) => {
    let i = 0; // стартовый индекс

    // Пропускаем строки пока не встретим строку содержащую "module" или "entry_func" => дошли до описания функций
    while (
      lines[i].indexOf("module") === -1 ||
      lines[i].indexOf("entry_func") === -1
    ) {
      ++i;
    }
    // Пока есть описание функции, то есть в строке содержатся слова "module" или "entry_func" итерируемся дальше и сохраняем
    // информацию в нужные объекты класса
    while (
      lines[i].indexOf("module") !== -1 ||
      lines[i].indexOf("entry_func") !== -1
    ) {
      let [func, metadata] = this.#parseMetadataFromString(lines[i]); // первый аргумент имя функции - ее имя,
      // второй аргумент - массив из 2-ух элементов, содержащий строки (module=<module_name>, entry_func=<entry_func_name>)
      this.functions[func] = {}; // у объекта functions создаем пустой объект по ключу func
      this.functions[func]["module"] = metadata[0].substring(
        // из полученного массива для первого элемента берем подстроку, которая начинается после знака = - это и есть module функции
        metadata[0].indexOf("=") + 1
      );
      this.functions[func]["entry_func"] = metadata[1].substring(
        // из полученного массива для второго элемента берем подстроку, которая начинается после знака = - это и есть entry_func функции
        metadata[1].indexOf("=") + 1
      );
      ++i;
    }

    // Пропускаем строки пока не встретим строку содержащую "module" или "entry_func" => дошли до описания предикатов
    while (
      lines[i].indexOf("module") === -1 ||
      lines[i].indexOf("entry_func") === -1
    ) {
      ++i;
    }
    // Пока есть описание предиката, то есть в строке содержатся слова "module" или "entry_func" итерируемся дальше и сохраняем
    // информацию в нужные объекты класса
    while (
      lines[i].indexOf("module") !== -1 ||
      lines[i].indexOf("entry_func") !== -1
    ) {
      let [predicate, meta] = this.#parseMetadataFromString(lines[i]); // первый аргумент имя предиката - ее имя,
      // второй аргумент - массив из 2-ух элементов, содержащий строки (module=<module_name>, entry_func=<entry_func_name>)
      this.predicates[predicate] = {}; // у объекта predicates создаем пустой объект по имени предиката
      this.predicates[predicate]["module"] = meta[0].substring(
        // из полученного массива для первого элемента берем подстроку, которая начинается после знака = - это и есть module предиката
        meta[0].indexOf("=") + 1
      );
      this.predicates[predicate]["entry_func"] = meta[1].substring(
        // из полученного массива для второго элемента берем подстроку, которая начинается после знака = - это и есть entry_func предиката
        meta[1].indexOf("=") + 1
      );
      ++i;
    }

    // Пропускаем строки пока не встретим строку содержащую слова "predicate" или "function" => дошли до описания ребер edges
    while (
      lines[i].indexOf("predicate") === -1 ||
      lines[i].indexOf("function") === -1
    ) {
      ++i;
    }
    // Пока есть описание ребра, то есть в строке содержатся слова "predicate" или "function" итерируемся дальше и сохраняем
    // информацию в нужные объекты класса
    while (
      lines[i].indexOf("predicate") !== -1 ||
      lines[i].indexOf("function") !== -1
    ) {
      let [edge, meta] = this.#parseMetadataFromString(lines[i]); // первый аргумент - имя ребра, второй аргумент - массив
      // из 2-х элементов, содержащий строки (predicate=<predicate_name>, function=<function_name>)
      this.edges[edge] = {}; // у объекта edges создаем пустой объект по имени ребра
      this.edges[edge]["predicate"] = meta[0].substring(
        // из полученного массива для первого элемента берем подстроку, которая начинается после знака = - это и есть
        // значение predicate для ребра
        meta[0].indexOf("=") + 1
      );
      this.edges[edge]["function"] = meta[1].substring(
        // из полученного массива для второго элемента берем подстроку, которая начинается после знака = - это и есть
        // значение function для ребра
        meta[1].indexOf("=") + 1
      );
      ++i;
    }
    return i;
  };

  /**
   * Приватная функция-член, которая парсит название функции и ее метаинформацию (module, entry_func)
   *
   * @param line - строка которую необходимо распарсить
   * @returns {[string, string[]]} название функции, массив содержащий 2 элемента: module, entry_func
   */
  #parseMetadataFromString = (line) => {
    const allowedCharacters = "abcdefghijklmnopqrstuvwxyz012345789_"; // допустимые символы для имени функции

    const currentLine = line.trim(); // убираем лишние пробелы вначале и в конце строки

    let i = 0; // индекс для итерации по строке
    let func = ""; // имя функции - пустая строка

    // Итерируемся пока не встретим недоступный символ
    while (allowedCharacters.indexOf(currentLine[i].toLowerCase()) !== -1) {
      func += currentLine[i]; // на каждой итерации добавляем к имени функции очередной прочитанный символ
      ++i;
    }

    // Парсим метаинформацию о функции, логика простая, по стандарту формата aDOT метаинформация представляется
    // в следующем виде:
    //    FUNC_1 [module=<module_name>,entry_func=<entry_func_name>]
    const meta = currentLine
      .substring(currentLine.indexOf("[") + 1, currentLine.length - 1) // берем подстроку, которая начинается
      // с открывающей квадратной скобки и заканчивается на закрывающей квадратной скобке
      .split(","); // разбираем строку на массив по символу запятой, таким образом всегда имеем два элемента:
    // 1-й module=<module_name>, 2-й entry_func=<entry_func_name>

    return [func, meta];
  };

  /**
   * Приватная функция-член, реализующая визуализацию вершин графа на холсте
   *
   * @param idx - индекс на котором закончилось описание метаинформации о графе
   * @param lines - массив строк, который содержит текст файла aDOT
   */
  #renderVertices = (idx, lines) => {
    // Начинаем визуализацию с того, что перезаписываем в массив lines только строки, которые касаются описания графа
    // Пока не встретим "->" или "=>" итерируемся по массиву строк
    while (lines[idx].indexOf("->") === -1 && lines[idx].indexOf("=>") === -1) {
      ++idx;
    }
    const startIdx = idx; // индекс строки в массиве в которой начинается описание графовой модели
    while (lines[idx].indexOf("->") !== -1 || lines[idx].indexOf("=>") !== -1) {
      ++idx;
    }
    const endIdx = idx; // индекс строки в массиве в которой заканчивается описание графовой модели
    lines = lines.slice(startIdx, endIdx); // перезаписываем массив lines, теперь массив lines хранит только описание
    // графовой модели

    let levels = {}; // объект для хранения уровней вершин
    let vertices = new Set();

    for (let j = 0; j < lines.length; ++j) {
      for (let i = 0; i < lines.length; ++i) {
        lines[i] = lines[i].trim();

        // Нашли стартовую вершину
        if (lines[i].indexOf("__BEGIN__") !== -1) {
          levels["1"] = []; // в объекте levels по ключу "1" создаем пустой массив
          levels["1"].push({
            // в массив добавляем название стартовой вершины
            [lines[i].substring(lines[i].lastIndexOf(" ") + 1)]: [], // название последней вершины находится легко: из строки
            // берем подстроку, которая начинается с последнего пробела, это и будет название нашей вершины
          });
          continue; // т.к. мы нашли стартовую вершину, мы инициализировали объект levels, выполнение далее идущего кода
          // логично только при наличии так называемого первого уровня (самая левая вершина графа)
        }

        // Нашли конечную вершину => просто переходим к следующей строке описания графовой модели, смысла обрабатывать эту
        // ситуацию отдельно нет, т.к. в ходе работы алгоритма, правая вершина и так окажется на самом последнем уровне,
        if (lines[i].indexOf("__END__") !== -1) {
          continue;
        }

        // Наша задача получить массив из двух элементов, 1-й элемент это вершина из которой выходит ребро, 2-й элемент
        // это вершина в которую приходит ребро (s1 => s2, parts[0] = s1, parts[1] = s2)
        const parts =
          lines[i].indexOf("->") !== -1
            ? lines[i].split("->")
            : lines[i].split("=>");
        const from = parts[0].trim(); // название вершины откуда пришло ребро
        const to = parts[1].trim().split(" ")[0]; // название вершины куда пришло ребро

        vertices.add(from);
        vertices.add(to);

        const fromVertexLevel = parseInt(this.#findLevel(levels, from)); // получаем уровень на котором располагается
        // вершина из которой выходит ребро
        // Допустима ситуация, когда мы встречаем связь вершин, но уровень вершины (вершина from) из которой выходит
        // ребро еще не определен, в таком случае из функции findLevel вернется undefined. Таким образом, если из функции
        // пришел undefined, то мы просто переходим в следующей строке
        if (!fromVertexLevel) {
          continue;
        }

        // Проверяем, что текущая связь между вершинами является циклом
        let isCycle = false; // флаг
        // Итерируемся по объекту levels
        Object.values(levels).forEach((level) => {
          // Для каждого уровня нам необходимо просмотреть массив объектов этого уровня
          level.forEach((vertex) => {
            // Для каждого элемента массива - это объект итерируемся по свойствам этого объекта и проверяем, что
            // текущее свойство равно полученной вершине from и что массив вершин по этому ключу содержит вершину to
            Object.keys(vertex).forEach((key) => {
              if (key === from && vertex[key].includes(to)) {
                // В таком случае мы нашли цикл, и нам не надо обрабатывать эти вершины отдельно, так как они уже
                // построены, а мы лишь встретили очередное ребро
                isCycle = true;
              }
            });
          });
        });
        if (isCycle) {
          continue;
        }

        /*this.#shortestPath(lines, "s1", to, "");
        this.#shortestPathIncludesVertex(lines, "s1", to, i.toString(), "");
        graph.pathShortest = "";
        graph.pathIncludesVertex = "";
        graph.pathShortestLength = 10000;
        graph.pathIncludesVertexLength = 10000;*/

        // Если у объекта levels отсутствует свойство fromVertexLevel + 1, то добавляем это свойство и пустой массив
        // как значение этого свойства
        if (!levels.hasOwnProperty(fromVertexLevel + 1)) {
          levels[fromVertexLevel + 1] = [];
        }

        // В этом цикле обрабатывается ситуация, когда несколько ребер приходят в одну вершину
        let flag = false; // флаг
        // Итерируемся по массиву, который является свойством по ключу fromVertexLevel + 1
        for (const element of levels[fromVertexLevel + 1]) {
          // Если элемент массива (напомним, что это объект) содержит свойство = to
          if (element.hasOwnProperty(to)) {
            // Если по этому свойству в массив ранее не добавлено вершины from, то добавляем ее
            if (!element[to].includes(from)) {
              element[to].push(from);
            }
            flag = true;
          }
        }

        // Если на предыдущем шаге не встретили такую вершину, то просто по ключу fromVertexLevel + 1 добавляем
        // новый элемент в массив, элемент представляет из себя объект с ключом to, и массивом по значению, массив
        // содержит один элемент - это вершина from
        if (!flag && !levels[fromVertexLevel + 1].hasOwnProperty(to)) {
          levels[fromVertexLevel + 1].push({ [to]: [from] });
        }
      }
    }

    // Проверяем корректность созданного объекта levels, более подробное описание - JSdoc к функции checkVerticesLevels
    const verticesWithNoneVisibility = this.#checkVerticesLevels(
      levels,
      vertices
    ); // получили объект, который содержит информацию
    // о вершинах которые не надо отрисовывать

    console.log("None visibility:", verticesWithNoneVisibility);

    // Находим уровень на котором расположено больше всего вершин, с него мы начинаем строить граф, влево и вправо
    let highest = []; // массив для хранения вершин самого "высокого" уровня
    let highestLevel = 1;
    Object.keys(levels).forEach((level) => {
      if (levels[level].length > highest.length) {
        highest = levels[level];
        highestLevel = level;
      }
    });

    let positions = {}; // Объект для хранения информации о построенных вершинах хранит: название вершины, позицию по
    // координате X, позицию по координате Y

    const startX = 40; // стартовая позиция по координате X - место где будет располагаться первая вершина
    const startY = 100; // "стартовая" позиция по координате Y. Заметим, что это не позиция первой вершины - это
    // позиция где будет располагаться самая верхняя вершина графа
    const horizontalOffset = 150; // горизонтальный сдвиг (расстояние между вершинами по координате X)
    const verticalOffset = 100; // вертикальный сдвиг (расстояние между вершинами по координате Y)

    // Начинаем отрисовку графа с отрисовки уровня с самым большим количеством вершин в нем
    let cx = startX + horizontalOffset * (parseInt(highestLevel) - 1); // координата X этого уровня
    let cy =
      startY - verticalOffset * verticesWithNoneVisibility[highestLevel].length; // координата Y равна стартовой позиции по координате Y
    // Итерируемся по массиву - строим вершины
    highest.forEach((vertex) => {
      Object.keys(vertex).forEach((current) => {
        if (!verticesWithNoneVisibility[highestLevel].includes(current)) {
          this.#addVertexFromADOT(cx, cy, current);
        }
        positions[current] = {
          // Ключ - название вершины, значение - позиция по координате X, позиция по координате Y
          x: cx,
          y: cy,
        };
      });
      cy += verticalOffset; // инкрементируем координату по Y на значение равное вертикальному сдвигу
    });

    // Строим вершины слева от самого высокого уровня
    let leftLevel = parseInt(highestLevel) - 1; // левый уровень
    let cxLeft = cx; // координата по X для левого подграфа
    while (leftLevel > 0) {
      // пока не просмотрим все уровни включая 1-й
      cxLeft -= horizontalOffset; // сдвигаем координату X на значение равное горизонтальному сдвигу
      for (let i = 0; i < levels[leftLevel].length; ++i) {
        // Итерируемся по массиву вершин текущего уровня
        Object.keys(levels[leftLevel][i]).forEach((value) => {
          // Для каждого элемента массива (вершины) находим список связанных с ней вершин
          let to = []; // массив для сохранения связанных вершин

          // Итерируемся по всем уровням и ищем связанные вершины
          Object.keys(levels).forEach((level) => {
            levels[level].forEach((arrayElement) => {
              Object.keys(arrayElement).forEach((vertex) => {
                arrayElement[vertex].forEach((connectedVertex) => {
                  if (connectedVertex === value) {
                    to.push(vertex);
                  }
                });
              });
            });
          });

          // Логика: найти координату Y самой верхней (минимум) связанной вершины, найти координату Y самой нижней
          // (максимум) связанной вершины
          let firstY = 10000, // инициализируем переменные для хранения минимума и максимума
            lastY = 0;
          to.forEach((vertex) => {
            const yPos = parseInt(positions[vertex]["y"]); // из объекта positions по названию связанной вершины
            // достаем координату по Y связанной вершины

            // Простой поиск минимума и максимума
            if (yPos > lastY) {
              lastY = yPos;
            }
            if (yPos < firstY) {
              firstY = yPos;
            }
          });

          cy = firstY + (lastY - firstY) / 2; // координата Y для текущей вершины - должна быть центрирована
          // по вертикали относительно связанных вершин

          // Строим вершину
          Object.keys(levels[leftLevel][i]).forEach((vertex) => {
            if (!verticesWithNoneVisibility[leftLevel].includes(vertex)) {
              this.#addVertexFromADOT(cxLeft, cy, vertex);
            }
            positions[vertex] = {
              x: cxLeft,
              y: cy,
            };
          });
        });
      }
      --leftLevel; // декрементируем уровень - спускаемся на уровень ниже
    }

    // Аналогичная логика и для построения вершин, которые располагаются справа от самого большого уровня
    let rightLevel = parseInt(highestLevel) + 1;
    let end = 0; // последний уровень, необходимо, чтобы остановить итерацию цикла while
    Object.keys(levels).forEach((key) => {
      if (parseInt(key) > end) {
        end = parseInt(key);
      }
    });

    let cxRight = cx;
    while (rightLevel <= end) {
      cxRight += horizontalOffset;
      for (let i = 0; i < levels[rightLevel].length; ++i) {
        Object.keys(levels[rightLevel][i]).forEach((key) => {
          let firstY = 10000,
            lastY = 0;

          levels[rightLevel][i][key].forEach((vertex) => {
            const yPos = parseInt(positions[vertex]["y"]);
            if (yPos > lastY) {
              lastY = yPos;
            }
            if (yPos < firstY) {
              firstY = yPos;
            }
          });

          cy = firstY + (lastY - firstY) / 2;

          if (!verticesWithNoneVisibility[rightLevel].includes(key)) {
            this.#addVertexFromADOT(cxRight, cy, key);
          }
          positions[key] = {
            x: cxRight,
            y: cy,
          };
        });
      }
      ++rightLevel;
    }
  };

  /**
   * Приватная функция-член, которая реализует проверку корректности объекта levels. После заполнения объекта levels может
   * получиться ситуация, когда, к примеру, вершина с меткой s9 располагается на 7-м уровне, а также на 4-м уровне.
   * Очевидно, что реальный уровень этой вершины - 7й, а то что эта вершина появилась на 4-м уровне - это издержки работы
   * алгоритма по заполнению объекта levels. Поэтому в этой функции мы делаем 2 основных действия:
   *  1) У вершины, которая располагается не на своем уровне, берем массив вершин из которых мы пришли в эту вершину
   *  и конкатенируем его с массивом такой же вершины, но которая располагается на своем уровне!
   *  2) Вершина, которая располагается не на своем месте, нам еще пригодится поэтому мы ее не удаляем из массива levels
   *  а только лишь сохраняем в объект, который будет хранить информацию о вершинах которые располагаются не на своих местах
   *
   * Также в функции реализована проверка уровня вершины(без дубликата), которая реализует перестановку вершины на подходящий уровень
   * если между этой вершиной и вершиной в которую идет ребро несколько уровней. В таком случае мы перетаскиваем эту
   * вершину на уровень где больше всего вершин (это необходимо для корректной отрисовки графа)
   *
   * @param levels - объект который содержит уровни вершин
   * @param vertices - коллекция вершин, которые используются в графовой модели
   * @returns {{}} - объект вершин которые необходимо скрыть при отрисовке
   */
  #checkVerticesLevels = (levels, vertices) => {
    let verticesWithNoneVisibility = {}; // Объект для хранения вершин, которые находятся не на своем уровне, в дальнейшем при
    // отрисовке вершин, мы будем проверять, что вершина отсутствует в этом объекте
    Object.keys(levels).forEach((level) => {
      // В объект verticesWithNoneVisibility добавляем ключи - которые являются уровнями вершин в графе, заметим, что
      // это скажем так инициализация, поскольку значением по ключу уровня является пустой массив
      verticesWithNoneVisibility[level] = [];
    });

    // Поиск дубликата вершины, который располагается на более раннем уровне

    for (const vertex of vertices) {
      const vertexLastLevel = this.#findLevel(levels, vertex);

      for (const level in levels) {
        if (level === vertexLastLevel) {
          break;
        }

        for (const elements of levels[level]) {
          for (const v in elements) {
            if (v === vertex) {
              console.log("Found vertex in incorrect level, vertex:", v);

              for (const e of levels[vertexLastLevel]) {
                if (e.hasOwnProperty(v)) {
                  e[v] = e[v].concat(elements[v]);
                }
              }

              verticesWithNoneVisibility[level].push(v);
            }
          }
        }
      }
    }

    // Поиск вершины, которая располагается на более чем 1 уровень раньше чем связанная с ней ребром вершина
    for (const level in levels) {
      for (const arrayElement of levels[level]) {
        for (const objectKey in arrayElement) {
          for (const fromVertex of arrayElement[objectKey]) {
            const fromVertexLevel = this.#findLevel(levels, fromVertex);
            if (level - fromVertexLevel > 1) {
              const highestLevel = this.#findHighestLevelBetween(
                levels,
                fromVertexLevel,
                level
              );

              let idx, el;
              for (let i = 0; i < levels[fromVertexLevel].length; ++i) {
                if (levels[fromVertexLevel][i].hasOwnProperty(fromVertex)) {
                  idx = i;
                  el = levels[fromVertexLevel][i];
                  levels[fromVertexLevel].splice(idx, 1);
                  levels[highestLevel].push(el);
                }
              }
            }
          }
        }
      }
    }

    return verticesWithNoneVisibility;
  };

  /**
   * Приватная функция-член, которая находит уровень переданной вершины. Логика довольно простая:
   *  1) Итерируемся по объекту levels - ключ это текущий уровень вершины
   *  2) По каждому ключу хранится массив, который хранит информацию о вершинах, которые принадлежат этому уровню
   *  3) Сам массив - это массив объектов, объект представляет собой следующую структуру:
   *      ключ - название вершины на этом уровне, значение - массив с названием вершин из которых мы пришли в эту вершину
   *
   *  Поэтому для нахождения уровня вершины, в п.2 достаточно итерироваться по массивам и для каждого элемента проверять,
   *  имеет ли он ключ from - это вершина, которую мы ищем. Осталось только вернуть из функции просматриваемый уровень
   *
   * @param levels - объект содержащий уровни вершин
   * @param from - название вершины уровень которой мы ищем
   * @returns {string} - найденный уровень вершины типа string, если мы не нашли вершину, то из функции вернется объект типа NaN
   */
  #findLevel = (levels, from) => {
    let lvl = undefined;
    Object.keys(levels).forEach((level) => {
      if (
        levels[level].filter((element) => element.hasOwnProperty(from)).length >
        0
      ) {
        lvl = level;
      }
    });

    return lvl;
  };

  /**
   * Приватная функция-член, которая ищет уровень с самым большим количеством вершин в нем, между левым и правым уровнями
   *
   * @param levels - объект содержащий уровни
   * @param leftLevel - левый уровень
   * @param rightLevel - правый уровень
   * @returns {string} - самый больший по количеству вершин уровень
   */
  #findHighestLevelBetween = (levels, leftLevel, rightLevel) => {
    let highest = levels[leftLevel].length;
    let highestLevel = leftLevel;
    for (const level in levels) {
      if (parseInt(level) < parseInt(leftLevel)) {
        continue;
      }
      if (parseInt(level) > parseInt(rightLevel)) {
        continue;
      }
      if (levels[level].length > highest) {
        highest = levels[level].length;
        highestLevel = level;
      }
    }

    return highestLevel;
  };

  /**
   * Приватная функция-член, которая реализует создание вершины при импорте из формата aDOT. Логика функции немного
   * отличается от логики создания вершины простым кликом по холсту, поэтому вынесена в отдельную функцию.
   *
   * @param positionX - центр вершины по оси X
   * @param positionY - центр вершины по оси Y
   * @param label - название (метка) вершины
   */
  #addVertexFromADOT = (positionX, positionY, label) => {
    const id = "vertex" + ++this.vertexID;
    const path = CanvasCreateVertex(
      positionX,
      positionY,
      id,
      this.radius,
      this.borderWidth
    );
    this.createdVerticesPositions.push([id, positionX, positionY]);
    this.createdVerticesLabels.add(label);
    const pathID = "vertex" + this.vertexID + "_path";
    const labelID = "vertex" + this.vertexID + "_label";
    CanvasCreateTextByPath(path, label, pathID, labelID, 50, this.radius / 10);
    this.vertices[id] = {};
    this.vertices[id]["edges"] = [];
    this.vertices[id]["metadata"] = {
      position: {
        x: positionX,
        y: positionY,
      },
      label: label,
      label_metadata: {
        pathID: pathID,
        labelID: labelID,
      },
    };
  };

  /**
   * Приватная функция-член, которая занимается рендерингом ребер при импорте из формата aDOT
   *
   * @param idx - индекс в массиве строк файла
   * @param lines - массив строк файла
   */
  #renderEdges = (idx, lines) => {
    // Пока не встретим строку в которой содержатся стрелки, инкрементируем индекс
    while (lines[idx].indexOf("->") === -1 && lines[idx].indexOf("=>") === -1) {
      ++idx;
    }
    const startIdx = idx; // стартовый индекс на элемент массива с которого начинается описание графовой модели
    // Пока есть описание графовой модели инкрементируем индекс
    while (lines[idx].indexOf("->") !== -1 || lines[idx].indexOf("=>") !== -1) {
      ++idx;
    }
    const endIdx = idx; // конечный индекс на элемент массива на котором заканчивается описание графовой модели
    lines = lines.slice(startIdx, endIdx); // обрезаем массив, чтобы осталось только описание графовой модели

    let skippedLines = []; // массив который будет хранить пропущенные линии, при парсинге ребер может получиться такая
    // ситуация что между вершинами есть цикл, но прямое ребро (без кривых Безье) еще не построено, поэтому отрисовку
    // этих ребер необходимо отложить и обработать уже после обработки всех вершин

    for (let line of lines) {
      // Пропускаем строки которые описывают стартовую и конченые вершины, при построении ребер они нас не интересуют
      if (line.indexOf("__BEGIN__") !== -1 || line.indexOf("__END__") !== -1) {
        continue;
      }
      line = line.trim(); // удаляем лишние пробелы в начале и в конце строки
      const parts = line.split(" "); // разбиваем строку по пробелам, получим 4 элемента массива
      const fromID = this.#getVertexIDbyName(parts[0].trim()); // 1-й элемент - вершина из которой выходит ребро
      const toID = this.#getVertexIDbyName(parts[2].trim()); // 3-й элемент - вершина в которую приходит ребро
      const edge = parts[3].trim(); // 4-й элемент - морфизм

      // Получаем координаты по X для вершины из которой выходит ребро и вершины в которую приходит ребро
      let fromX = 0,
        toX = 0;
      Object.keys(this.vertices).forEach((vertex) => {
        if (vertex === fromID) {
          fromX = parseInt(this.vertices[vertex]["metadata"]["position"]["x"]);
        }
        if (vertex === toID) {
          toX = parseInt(this.vertices[vertex]["metadata"]["position"]["x"]);
        }
      });

      // Если вершина в которую приходит ребро находится левее чем вершина из которой выходит ребро, то вероятно это
      // цикл и стоит отложить отрисовку этих ребер, поэтому мы сохраняем их в отдельный массив
      console.log("toX", toX, "fromX", fromX);
      if (toX < fromX) {
        skippedLines.push(line);
      } else {
        this.#addEdgeFromADOT(fromID, toID, edge);
      }
    }

    // Отрисовка отложенных ребер
    for (const line of skippedLines) {
      const parts = line.split(" ");
      const fromID = this.#getVertexIDbyName(parts[0].trim()); // 1-й элемент - вершина из которой выходит ребро
      const toID = this.#getVertexIDbyName(parts[2].trim()); // 3-й элемент - вершина в которую приходит ребро
      const edge = parts[3].trim(); // 4-й элемент - морфизм
      let fromX = 0,
        toX = 0;
      Object.values(this.vertices).forEach((vertex) => {
        if (vertex["metadata"]["label"] === fromID) {
          fromX = parseInt(vertex["metadata"]["position"]["x"]);
        }
        if (vertex["metadata"]["label"] === toID) {
          toX = parseInt(vertex["metadata"]["position"]["x"]);
        }
      });
      this.#addEdgeFromADOT(fromID, toID, edge);
    }
  };

  /**
   * Приватная функция-член, которая реализует отрисовку и сохранение информации о ребрах
   *
   * @param fromVertexID - название вершины откуда выходит ребро
   * @param toVertexID - название вершины куда приходит ребро
   * @param edge - edge графа
   * @returns {boolean} - успешно/неуспешно построено
   */
  #addEdgeFromADOT = (fromVertexID, toVertexID, edge) => {
    if (fromVertexID === toVertexID) {
      setInformationConsoleMessage(
        "Синтаксическая ошибка. Вы не можете построить из вершины ребро в нее саму"
      );
      return false;
    }
    const amount = this.#countEdgesAmount(fromVertexID, toVertexID);

    const x1 = this.vertices[fromVertexID]["metadata"]["position"]["x"];
    const y1 = this.vertices[fromVertexID]["metadata"]["position"]["y"];
    const x2 = this.vertices[toVertexID]["metadata"]["position"]["x"];
    const y2 = this.vertices[toVertexID]["metadata"]["position"]["y"];

    const id = "edge" + ++this.edgeID;
    let path, edgeType;
    if (amount === 0) {
      path = CanvasCreateStraightEdge(
        x1,
        y1,
        x2,
        y2,
        this.radius,
        id,
        this.arrowheadSize
      );
      edgeType = "straight";
    } else {
      path = CanvasCreateBezierEdge(
        x1,
        y1,
        x2,
        y2,
        this.radius,
        id,
        this.arrowheadSize,
        amount * 3
      );
      this.vertices[fromVertexID]["cycle"] = true;
      edgeType = "bezier";
    }
    const e = edge.substring(edge.indexOf("=") + 1, edge.indexOf("]"));
    let predicate, func;
    for (const currentEdge in this.edges) {
      if (currentEdge === e) {
        predicate = this.edges[currentEdge]["predicate"];
        func = this.edges[currentEdge]["function"];
      }
    }
    const label = `<${predicate}, ${func}>`;
    const pathID = "edge" + this.edgeID + "_path";
    const labelID = "edge" + this.edgeID + "_label";
    CanvasCreateTextByPath(path, label, pathID, labelID, 50, -10);

    this.vertices[fromVertexID]["edges"].push(
      // Для первой вершины
      {
        value: toVertexID,
        direction: "from",
        label: label,
        type: edgeType,
        predicate: predicate,
        function: func,
        metadata: {
          edgeID: id,
          pathID: pathID,
          labelID: labelID,
        },
      }
    );
    this.vertices[toVertexID]["edges"].push({
      value: fromVertexID,
      direction: "to",
      label: label,
      type: edgeType,
      predicate: predicate,
      function: func,
      metadata: {
        edgeID: id,
        pathID: pathID,
        labelID: labelID,
      },
    });
  };

  /**
   * Приватная функция-член, которая ищет ID вершины по ее названию
   *
   * @param name - название
   * @returns {string} - ID
   */
  #getVertexIDbyName = (name) => {
    let id;
    for (const vertex in this.vertices) {
      if (this.vertices[vertex]["metadata"]["label"] === name) {
        id = vertex;
      }
    }

    return id;
  };

  /**
   * Приватная функция-член, для парсинга параллелизма
   *
   * @param lines - массив строк файла в формате aDOT
   */
  #parseParallelism = (lines) => {
    let idx = 0;
    while (
      lines[idx].indexOf("parallelism") === -1 &&
      idx !== lines.length - 1
    ) {
      ++idx;
    }
    while (lines[idx].indexOf("parallelism") !== -1) {
      const parts = lines[idx].split(" ");
      const label = parts[0].trim();
      const parallelismMethod = parts[1].substring(
        parts[1].indexOf("=") + 1,
        parts[1].indexOf("]")
      );
      for (const vertex in this.vertices) {
        if (this.vertices[vertex]["metadata"]["label"] === label) {
          this.vertices[vertex]["parallelism"] = parallelismMethod;
        }
      }
      ++idx;
    }
  };

  #shortestPath(lines, from, to, indexes) {
    if (indexes.length >= lines.length * 2) {
      return;
    }
    for (let i = 0; i < lines.length; ++i) {
      const members = lines[i].trim().split(" ");
      if (members[0] === from) {
        if (members[2] === to) {
          if ((indexes + i).length < graph.pathShortestLength) {
            graph.pathShortest = (indexes + i).split(" ");
            graph.pathShortestLength = (indexes + i).length;
          }
          return;
        }
        this.#shortestPath(lines, members[2], to, indexes + i + " ");
      }
    }
  }

  #shortestPathIncludesVertex(lines, from, to, includes, indexes) {
    if (indexes.length >= lines.length * 2) {
      return;
    }
    for (let i = 0; i < lines.length; ++i) {
      const members = lines[i].trim().split(" ");
      if (members[0] === from) {
        if (members[2] === to) {
          const pathMembers = (indexes + i).split(" ");
          if (
            pathMembers.lastIndexOf(includes) === pathMembers.length - 1 &&
            pathMembers.length < graph.pathIncludesVertexLength
          ) {
            graph.pathIncludesVertex = pathMembers;
            graph.pathIncludesVertexLength = pathMembers.length;
            return;
          }
        }
        this.#shortestPathIncludesVertex(
          lines,
          members[2],
          to,
          includes,
          indexes + i + " "
        );
      }
    }
  }
}

let graph = new Graph();
let actionsDisabled = false;

let lastButtonSelected;

const setVertexHandler = (event) => {
  const rect = svgCanvas.getBoundingClientRect();
  const centerX = event.clientX - rect.left;
  const centerY = event.clientY - rect.top;
  if (graph.AddVertex(centerX, centerY)) {
    svgCanvas.removeEventListener("click", setVertexHandler);
  }
};
const setVertexesHandler = () => {
  if (!actionsDisabled) {
    svgCanvas.removeEventListener("click", deleteVertexHandler);
    svgCanvas.removeEventListener("click", setEdgeHandler);
    svgCanvas.removeEventListener("click", deleteEdgeHandler);
    svgCanvas.removeEventListener("click", determineStartAndEndVertices);

    DOMChangeColor(lastButtonSelected);
    DOMChangeColor(SetVertexButton, "#cccccc");
    lastButtonSelected = SetVertexButton;
    setInformationConsoleMessage(
      `Вы выбрали 'Добавить вершину', кликните на холст, чтобы добавить вершину`
    );

    svgCanvas.addEventListener("click", setVertexHandler);
  }
};
SetVertexButton.addEventListener("click", setVertexesHandler);

const deleteVertexHandler = (event) => {
  if (event.target.closest(".vertex")) {
    const id = event.target.getAttribute("id");
    graph.DeleteVertex(id);
    svgCanvas.removeEventListener("click", deleteVertexHandler);
  }
};
const deleteVertexesHandler = () => {
  if (!actionsDisabled) {
    svgCanvas.removeEventListener("click", setVertexHandler);
    svgCanvas.removeEventListener("click", setEdgeHandler);
    svgCanvas.removeEventListener("click", deleteEdgeHandler);
    svgCanvas.removeEventListener("click", determineStartAndEndVertices);

    DOMChangeColor(lastButtonSelected);
    DOMChangeColor(DeleteVertexButton, "#cccccc");
    lastButtonSelected = DeleteVertexButton;
    setInformationConsoleMessage(
      `Вы выбрали 'Удалить вершину', кликните на вершину, чтобы ее удалить`
    );

    svgCanvas.addEventListener("click", deleteVertexHandler);
  }
};
DeleteVertexButton.addEventListener("click", deleteVertexesHandler);

let id1 = "",
  id2 = "";
const setEdgeHandler = (event) => {
  if (event.target.closest(".vertex")) {
    if (id1 === "") {
      // Select first vertex
      id1 = event.target.getAttribute("id");
      CanvasChangeBorderColor([id1], "#B92808");
      setInformationConsoleMessage(`Выберите вторую вершину`);
      actionsDisabled = true;
    } else {
      id2 = event.target.getAttribute("id");

      if (graph.AddEdge(id1, id2)) {
        id1 = id2 = "";
        svgCanvas.removeEventListener("click", setEdgeHandler);
      } else {
        actionsDisabled = false;
        id1 = id2 = "";
      }
    }
  }
};
const setEdgesHandler = () => {
  if (!actionsDisabled) {
    svgCanvas.removeEventListener("click", setVertexHandler);
    svgCanvas.removeEventListener("click", deleteVertexHandler);
    svgCanvas.removeEventListener("click", deleteEdgeHandler);
    svgCanvas.removeEventListener("click", determineStartAndEndVertices);

    DOMChangeColor(lastButtonSelected);
    DOMChangeColor(SetEdgeButton, "#cccccc");
    lastButtonSelected = SetEdgeButton;
    setInformationConsoleMessage(
      `Вы выбрали 'Добавить ребро', выберите первую вершину`
    );

    svgCanvas.addEventListener("click", setEdgeHandler);
  }
};
SetEdgeButton.addEventListener("click", setEdgesHandler);

const deleteEdgeHandler = (event) => {
  if (event.target.closest(".edge")) {
    const id = event.target.getAttribute("id");
    graph.DeleteEdge(id);
  }
};
const deleteEdgesHandler = () => {
  if (!actionsDisabled) {
    svgCanvas.removeEventListener("click", setVertexHandler);
    svgCanvas.removeEventListener("click", deleteVertexHandler);
    svgCanvas.removeEventListener("click", setEdgeHandler);
    svgCanvas.removeEventListener("click", determineStartAndEndVertices);

    DOMChangeColor(lastButtonSelected);
    DOMChangeColor(DeleteEdgeButton, "#cccccc");
    lastButtonSelected = DeleteEdgeButton;
    setInformationConsoleMessage(
      `Вы выбрали 'Удалить ребро', кликните на ребро, чтобы его удалить`
    );

    svgCanvas.addEventListener("click", deleteEdgeHandler);
  }
};
DeleteEdgeButton.addEventListener("click", deleteEdgesHandler);

let startVertex = undefined,
  endVertex = undefined;
const determineStartAndEndVertices = (event) => {
  if (event.target.closest(".vertex")) {
    if (startVertex === undefined) {
      startVertex = event.target.getAttribute("id");
      CanvasChangeBorderColor([startVertex], "#B92808");
      setInformationConsoleMessage("Выберите конечную вершину кликну на нее");
      actionsDisabled = true;
    } else {
      endVertex = event.target.getAttribute("id");

      if (graph.ExportADOT(startVertex, endVertex)) {
        startVertex = endVertex = undefined;
        actionsDisabled = false;
        svgCanvas.removeEventListener("click", determineStartAndEndVertices);
      } else {
        actionsDisabled = false;
        startVertex = endVertex = undefined;
      }
    }
  }
};
const exportHandler = () => {
  if (!actionsDisabled) {
    svgCanvas.removeEventListener("click", setVertexHandler);
    svgCanvas.removeEventListener("click", deleteVertexHandler);
    svgCanvas.removeEventListener("click", setEdgeHandler);
    svgCanvas.removeEventListener("click", deleteEdgeHandler);
    svgCanvas.removeEventListener("click", determineStartAndEndVertices);

    DOMChangeColor(lastButtonSelected);
    DOMChangeColor(ExportADOTButton, "#cccccc");
    lastButtonSelected = ExportADOTButton;
    setInformationConsoleMessage("Выберите стартовую вершину кликнув на нее");
    svgCanvas.addEventListener("click", determineStartAndEndVertices);
  }
};
ExportADOTButton.addEventListener("click", exportHandler);

const fileToText = (file, callback) => {
  const reader = new FileReader();
  reader.readAsText(file);
  reader.onload = () => {
    callback(reader.result);
  };
};
const UploadHandler = () => {
  DOMChangeColor(lastButtonSelected);
  const file = ImportADOTButton.files.item(0);
  if (file) {
    fileToText(file, (text) => {
      graph.ImportADOT(text);
    });
  }
};
ImportADOTButton.addEventListener("change", UploadHandler);

const ClearCanvasHandler = () => {
  if (!actionsDisabled) {
    svgCanvas.removeEventListener("click", setVertexHandler);
    svgCanvas.removeEventListener("click", deleteVertexHandler);
    svgCanvas.removeEventListener("click", setEdgeHandler);
    svgCanvas.removeEventListener("click", deleteEdgeHandler);
    svgCanvas.removeEventListener("click", determineStartAndEndVertices);

    DOMChangeColor(lastButtonSelected);
    graph.Clear();
  }
};
ClearCanvasButton.addEventListener("click", ClearCanvasHandler);

const FindCyclesHandler = () => {
  if (!actionsDisabled) {
    svgCanvas.removeEventListener("click", setVertexHandler);
    svgCanvas.removeEventListener("click", deleteVertexHandler);
    svgCanvas.removeEventListener("click", setEdgeHandler);
    svgCanvas.removeEventListener("click", deleteEdgeHandler);
    svgCanvas.removeEventListener("click", determineStartAndEndVertices);

    DOMChangeColor(lastButtonSelected);
    graph.FindCycles();
  }
};
FindCyclesButton.addEventListener("click", FindCyclesHandler);

/**
 * Функция для создания вершины на холсте
 *
 * @param x - центр по координате X
 * @param y - центр по координате Y
 * @param id - id
 * @param radius - радиус
 * @param borderWidth - толщина границы
 * @returns {*} - объект d3.path() - мнимая линия - горизонтальная хорда через центр окружности, необходимо для построения
 * текста метки вершины
 */
const CanvasCreateVertex = (x, y, id, radius, borderWidth) => {
  // Создание круга на холсте, добавляем тег <circle> и описываем необходимые свойства
  svg
    .append("circle")
    .attr("cx", x)
    .attr("cy", y)
    .attr("r", radius)
    .attr("strokeWidth", borderWidth)
    .attr("id", id)
    .attr("fill", "#FFFFFF")
    .attr("stroke", "#000000")
    .attr("class", "vertex"); // для всех вершин добавляем класс vertex, чтобы в дальнейшем делать выборку всех
  // вершин через класс "vertex"

  // Строим мнимую линию - горизонтальную хорду, проходящую через центр окружности
  const path = d3.path();
  path.moveTo(x - radius, y);
  path.lineTo(x + radius, y);
  return path;
};

/**
 * Функция для построения текста по переданному пути (path) на холсте
 *
 * @param path - путь по которому будет строиться текст
 * @param text - текст
 * @param pathID - ID пути
 * @param textID - ID текста
 * @param offset - горизонтальный сдвиг от начала пути
 * @param topOffset - вертикальный сдвиг от центра пути (текст выравнивается вертикально по центру мнимой линии,
 * этот параметр необходим, чтобы сдвинуть текст немного ниже по вертикали, для более красиво визуализации)
 */
const CanvasCreateTextByPath = (
  path,
  text,
  pathID,
  textID,
  offset,
  topOffset = 0
) => {
  // Добавляем на холст тег <path> с необходимым свойствами
  svg.append("path").attr("d", path).attr("id", pathID).style("fill", "none");

  // Добавляем на холст тег <text> с необходимыми свойствами
  svg
    .append("text")
    .attr("id", textID)
    .attr("dy", topOffset)
    .append("textPath") // Добавляем тег <textPath> - строим текст по пути (path)
    .attr("xlink:href", "#" + pathID)
    .style("text-anchor", "middle")
    .attr("startOffset", offset + "%")
    .attr("id", textID)
    .text(text);
};

/**
 * Функция для построения прямого ребра на холсте
 *
 * @param x1 - центр вершины из которой выходит ребро по координате X
 * @param y1 - центр вершины из которой выходит ребро по координате Y
 * @param x2 - центр вершины в которую приходит ребро по координате X
 * @param y2 - центр вершины в которую приходит ребро по координате Y
 * @param radius - радиус вершины
 * @param id - ID ребра
 * @param arrowheadSize - размер стрелки (что это за значение подробнее описано в конструкторе класса)
 * @returns {*} - путь по которому строилась стрелка, необходим для дальнейшего построения метки ребра по этому пути
 */
const CanvasCreateStraightEdge = (
  x1,
  y1,
  x2,
  y2,
  radius,
  id,
  arrowheadSize
) => {
  // Вычисляем координаты начала и конца линии
  const [fromX, fromY, toX, toY] = calcLineCoordinatesBetweenVertices(
    x1,
    y1,
    x2,
    y2,
    radius
  );

  // Вычисляем координаты "усов" стрелки
  const [endArrow1X, endArrow1Y, endArrow2X, endArrow2Y] =
    calcArrowheadCoordinates(fromX, fromY, toX, toY, arrowheadSize);

  // Строим путь по которому в дальнейшем будет строиться линия соединяющая вершины
  const path = d3.path();
  path.moveTo(fromX, fromY);
  path.lineTo(toX, toY);
  path.moveTo(toX, toY);
  path.lineTo(endArrow1X, endArrow1Y);
  path.moveTo(toX, toY);
  path.lineTo(endArrow2X, endArrow2Y);

  // Создаем тег <path> и определяем у него необходимые свойства
  svg
    .append("path")
    .attr("stroke", "black")
    .attr("d", path)
    .attr("id", id)
    .attr("class", "edge");

  // Получаем d3.path() по которому в дальнейшем будет строиться метка для ребра
  const textPath = d3.path();
  if (x2 < x1) {
    // Проверка необходима, чтобы path строился как бы слева->направо, это нужно, чтобы текст всегда
    // находился выше стрелки, вне зависимости от расположения вершин
    textPath.moveTo(toX, toY);
    textPath.lineTo(fromX, fromY);
    return textPath;
  }

  textPath.moveTo(fromX, fromY);
  textPath.lineTo(toX, toY);
  return textPath;
};

/**
 * Функция для построения ребра с использованием кривых Безье
 *
 * @param x1 - центр вершины из которой выходит ребро по координате X
 * @param y1 - центр вершины из которой выходит ребро по координате Y
 * @param x2 - центр вершины в которую приходит ребро по координате X
 * @param y2 - центр вершины в которую приходит ребро по координате Y
 * @param radius - радиус вершины
 * @param id - ID ребра
 * @param arrowheadSize - размер стрелки (подробнее описано в конструкторе класса Graph)
 * @param coefficient - коэффициент для точек по которым будет строиться кривая Безье
 * @returns {*} - путь по которому строилась стрелка, необходим для дальнейшего построения метки ребра по этому пути
 */
const CanvasCreateBezierEdge = (
  x1,
  y1,
  x2,
  y2,
  radius,
  id,
  arrowheadSize,
  coefficient
) => {
  // Координаты начала и конца мнимой прямой линии между вершинами
  const [fromX, fromY, toX, toY] = calcLineCoordinatesBetweenVertices(
    x1,
    y1,
    x2,
    y2,
    radius
  );

  // Вычисление координат из которых будет выходить линия
  let size = arrowheadSize * 3;
  const [temp2X, temp2Y] = calcArrowheadCoordinates(
    toX,
    toY,
    fromX,
    fromY,
    size
  ).slice(2);
  const [startX, startY] = calcArrowheadCoordinates(
    fromX,
    fromY,
    temp2X,
    temp2Y,
    size * 1.3
  ).slice(0, 2);

  // Вычисление координат в которую будет приходить линия
  const [temp1X, temp1Y] = calcArrowheadCoordinates(
    fromX,
    fromY,
    toX,
    toY,
    size
  ).slice(0, 2);
  const [endX, endY] = calcArrowheadCoordinates(
    toX,
    toY,
    temp1X,
    temp1Y,
    size * 1.3
  ).slice(2);

  // Вычисление координат первой точки для кривой Безье
  size = arrowheadSize * coefficient * 2;
  const [temp3X, temp3Y] = calcArrowheadCoordinates(
    toX,
    toY,
    fromX,
    fromY,
    size
  ).slice(2);
  const [bezier1X, bezier1Y] = calcArrowheadCoordinates(
    fromX,
    fromY,
    temp3X,
    temp3Y,
    size
  ).slice(0, 2);

  // Вычисление координат второй точки для кривой Безье
  const [temp4X, temp4Y] = calcArrowheadCoordinates(
    fromX,
    fromY,
    toX,
    toY,
    size
  ).slice(0, 2);
  const [bezier2X, bezier2Y] = calcArrowheadCoordinates(
    toX,
    toY,
    temp4X,
    temp4Y,
    size
  ).slice(2);

  // Построение наконечника стрелки
  if (coefficient / 3 === 1) {
    // Если это первая кривая Безье, то строим наконечник, иначе нет
    size = arrowheadSize * coefficient * 1.25;
    const [temp6X, temp6Y] = calcArrowheadCoordinates(
      fromX,
      fromY,
      toX,
      toY,
      size
    ).slice(0, 2);
    const [arrowStart2X, arrowStart2Y] = calcArrowheadCoordinates(
      toX,
      toY,
      temp6X,
      temp6Y,
      size
    ).slice(2);

    // Координаты "усов" стрелки
    const [endArrow1X, endArrow1Y, endArrow2X, endArrow2Y] =
      calcArrowheadCoordinates(
        arrowStart2X,
        arrowStart2Y,
        endX,
        endY,
        arrowheadSize
      );

    // Путь по которому строится линия
    const path = d3.path();
    path.moveTo(startX, startY);
    path.bezierCurveTo(bezier1X, bezier1Y, bezier2X, bezier2Y, endX, endY);
    path.moveTo(endX, endY);
    path.lineTo(endArrow1X, endArrow1Y);
    path.moveTo(endX, endY);
    path.lineTo(endArrow2X, endArrow2Y);

    // Добавляем тег <path> и определяем необходимые свойства
    svg
      .append("path")
      .attr("stroke", "black")
      .attr("fill", "transparent")
      .attr("d", path)
      .attr("id", id)
      .attr("class", "edge");
  } else {
    // Наконечник стрелки не строим, поэтому сразу получаем путь по которому будет строиться линия
    const path = d3.path();
    path.moveTo(startX, startY);
    path.bezierCurveTo(bezier1X, bezier1Y, bezier2X, bezier2Y, endX, endY);
    path.moveTo(endX, endY);

    // Добавляем тег <path> и определяем необходимые свойства
    svg
      .append("path")
      .attr("stroke", "black")
      .attr("fill", "transparent")
      .attr("d", path)
      .attr("id", id)
      .attr("class", "edge");
  }

  // Получаем d3.path() по которому в дальнейшем будет строиться метка для ребра
  if (x2 < x1) {
    // Проверка необходима, чтобы path строился как бы слева->направо, это нужно, чтобы текст всегда
    // находился выше стрелки, вне зависимости от расположения вершин
    const textPath = d3.path();
    textPath.moveTo(endX, endY);
    textPath.bezierCurveTo(
      bezier2X,
      bezier2Y,
      bezier1X,
      bezier1Y,
      startX,
      startY
    );
    return textPath;
  }
  const textPath = d3.path();
  textPath.moveTo(startX, startY);
  textPath.bezierCurveTo(bezier1X, bezier1Y, bezier2X, bezier2Y, endX, endY);
  return textPath;
};

/**
 * Функция для вычисления координат "усов" стрелки
 *
 * @param fromX - координата X откуда вышла стрелка
 * @param fromY - координата Y откуда вышла стрелка
 * @param toX - координата X куда пришла стрелка
 * @param toY - координата Y куда пришла стрелка
 * @param arrowheadSize - размер стрелки (подробнее описано в конструкторе класса Graph)
 * @returns {[number, number, number, number]} - координаты усов стрелки [X1, Y1, X2, Y2], где индекс это номер "уса"
 */
const calcArrowheadCoordinates = (fromX, fromY, toX, toY, arrowheadSize) => {
  // Алгебра + геометрия - в комментариях излишне
  const length = Math.sqrt((toX - fromX) ** 2 + (toY - fromY) ** 2);
  const unitDx = (toX - fromX) / length;
  const unitDy = (toY - fromY) / length;
  const endArrow1X = toX - unitDx * arrowheadSize - unitDy * arrowheadSize;
  const endArrow1Y = toY - unitDy * arrowheadSize + unitDx * arrowheadSize;
  const endArrow2X = toX - unitDx * arrowheadSize + unitDy * arrowheadSize;
  const endArrow2Y = toY - unitDy * arrowheadSize - unitDx * arrowheadSize;

  return [endArrow1X, endArrow1Y, endArrow2X, endArrow2Y];
};

/**
 * Функция для вычисления координат прямой линии между вершинами
 *
 * @param x1 - координата X откуда вышла линия
 * @param y1 - координата Y откуда вышла линия
 * @param x2 - координата X куда пришла линия
 * @param y2 - координата Y куда пришла линия
 * @param radius - радиус вершины
 * @returns {[*, *, *, *]} - координаты начала и конца стрелки [X1, Y1, X2, Y2], 1 - индекс откуда выходит линия, 2 -
 * индекс куда приходит линия
 */
const calcLineCoordinatesBetweenVertices = (x1, y1, x2, y2, radius) => {
  const legX = Math.abs(x2 - x1);
  const legY = Math.abs(y2 - y1);
  const hypotenuse = Math.sqrt(legX ** 2 + legY ** 2);
  const fromX = x1 + ((x2 - x1) * radius) / hypotenuse;
  const fromY = y1 + ((y2 - y1) * radius) / hypotenuse;
  const toX = x1 + ((x2 - x1) * (hypotenuse - radius)) / hypotenuse;
  const toY = y1 + ((y2 - y1) * (hypotenuse - radius)) / hypotenuse;

  return [fromX, fromY, toX, toY];
};

/**
 * Функция для установки текст в нижний сайдбар (консоль)
 *
 * @param message - текст сообщения
 */
const setInformationConsoleMessage = (message) => {
  InformationConsoleMessage.textContent = message;
};

/**
 * Функция для рендеринга поля ввода в правом сайдбаре
 */
const renderVertexInput = () => {
  // Получаем объект блока правого сайдбара
  const rightSidebarBlock = document.querySelector(".right-sidebar-block");

  // Очищаем правый сайдбар
  clearRightSidebar(rightSidebarBlock);

  // Добавляем необходимые теги и назначаем им необходимые свойства
  const input = document.createElement("input");
  const label = document.createElement("label");
  rightSidebarBlock.insertBefore(input, RightSidebarConfirmButton);
  rightSidebarBlock.insertBefore(label, input);

  // className
  label.className = "right-sidebar__label";
  input.className = "right-sidebar__input";

  // textContent
  label.textContent = "Label: ";

  // ID
  input.id = "right-sidebar__input__id";
};

/**
 * Функция для рендеринга полей ввода предиката и функции для ребра
 *
 * @param needParallelismSelection - true - необходимо зарендерить <select> для выбора типа параллелизма
 */
const renderEdgeInputFields = (needParallelismSelection) => {
  // Получаем объект блока правого сайдбара
  const rightSidebarBlock = document.querySelector(".right-sidebar-block");

  // Очищаем правый сайдбар
  clearRightSidebar(rightSidebarBlock);

  // Добавляем необходимые теги и назначаем им необходимые свойства
  const input1 = document.createElement("input");
  const input2 = document.createElement("input");
  const label1 = document.createElement("label");
  const label2 = document.createElement("label");

  rightSidebarBlock.insertBefore(input2, RightSidebarConfirmButton);
  rightSidebarBlock.insertBefore(label2, input2);
  rightSidebarBlock.insertBefore(input1, label2);
  rightSidebarBlock.insertBefore(label1, input1);

  // Необходим выбор типа параллелизма => рендерим тег <select> для выбора типа
  if (needParallelismSelection) {
    const selector = document.createElement("select");
    const multiThreading = document.createElement("option");
    const pseudoParallelism = document.createElement("option");

    selector.appendChild(multiThreading);
    selector.appendChild(pseudoParallelism);
    rightSidebarBlock.insertBefore(selector, RightSidebarConfirmButton);

    // className
    selector.className = "bottomSelector";

    // ID
    selector.id = "selectorParallelism_id";

    // value
    multiThreading.value = "threading";
    pseudoParallelism.value = "pseudo-parallelism";

    // text
    multiThreading.text = "threading";
    pseudoParallelism.text = "pseudo-parallelism";
  }

  // className
  label2.className = "right-sidebar__label";
  label1.className = "right-sidebar__label";
  input2.className = "right-sidebar__input";
  input1.className = "right-sidebar__input";

  // textContent
  label2.textContent = "Function:";
  label1.textContent = "Predicate:";

  // ID
  input2.id = "right-sidebar__input-function__id";
  input1.id = "right-sidebar__input-predicate__id";

  // Value
  input1.value = "";
  input2.value = "";
};

/**
 * Функция для рендеринга полей ввода информации о предикате и функции
 *
 * @param isNewPredicate - true/false - если true, то рендерим поля ввода для ввода информации о новом предикате
 * @param isNewFunction - true/false - если true, то рендерим поля ввода для ввода информации о новой функции
 */
const renderEdgeInputFieldPredicateFunction = (
  isNewPredicate,
  isNewFunction
) => {
  // Получаем объект блока правого сайдбара
  const rightSidebarBlock = document.querySelector(".right-sidebar-block");

  // Очищаем правый сайдбар
  clearRightSidebar(rightSidebarBlock);

  // Добавляем поля для ввода module и entry_func
  if (isNewPredicate) {
    const headerPredicate = document.createElement("p");
    const inputModule = document.createElement("input");
    const labelModule = document.createElement("label");
    const inputEntryFunc = document.createElement("input");
    const labelEntryFunc = document.createElement("label");

    rightSidebarBlock.insertBefore(headerPredicate, RightSidebarConfirmButton);
    rightSidebarBlock.insertBefore(labelModule, RightSidebarConfirmButton);
    rightSidebarBlock.insertBefore(inputModule, RightSidebarConfirmButton);
    rightSidebarBlock.insertBefore(labelEntryFunc, RightSidebarConfirmButton);
    rightSidebarBlock.insertBefore(inputEntryFunc, RightSidebarConfirmButton);

    // className
    headerPredicate.className = "right-sidebar__label";
    labelEntryFunc.className = "right-sidebar__label";
    labelModule.className = "right-sidebar__label";
    inputModule.className = "right-sidebar__input";
    inputEntryFunc.className = "right-sidebar__input";

    // textContent
    headerPredicate.textContent = "Predicate:";
    labelEntryFunc.textContent = "Entry function:";
    labelModule.textContent = "Entry module: ";

    // ID
    inputModule.id = "right-sidebar__input-predicate-module__id";
    inputEntryFunc.id = "right-sidebar__input-predicate-entry__id";

    // value
    inputModule.value = "DEFAULT_VALUE";
    inputEntryFunc.value = "DEFAULT_VALUE";
  }

  if (isNewFunction) {
    const headerFunction = document.createElement("p");
    const inputModule = document.createElement("input");
    const labelModule = document.createElement("label");
    const inputEntryFunc = document.createElement("input");
    const labelEntryFunc = document.createElement("label");

    rightSidebarBlock.insertBefore(headerFunction, RightSidebarConfirmButton);
    rightSidebarBlock.insertBefore(labelModule, RightSidebarConfirmButton);
    rightSidebarBlock.insertBefore(inputModule, RightSidebarConfirmButton);
    rightSidebarBlock.insertBefore(labelEntryFunc, RightSidebarConfirmButton);
    rightSidebarBlock.insertBefore(inputEntryFunc, RightSidebarConfirmButton);

    // className
    headerFunction.className = "right-sidebar__label";
    labelEntryFunc.className = "right-sidebar__label";
    labelModule.className = "right-sidebar__label";
    inputModule.className = "right-sidebar__input";
    inputEntryFunc.className = "right-sidebar__input";

    // textContent
    headerFunction.textContent = "Function:";
    labelEntryFunc.textContent = "Entry function:";
    labelModule.textContent = "Entry module: ";

    // ID
    inputModule.id = "right-sidebar__input-function-module__id";
    inputEntryFunc.id = "right-sidebar__input-function-entry__id";

    // value
    inputModule.value = "DEFAULT_VALUE";
    inputEntryFunc.value = "DEFAULT_VALUE";
  }
};

/**
 * Функция для очистки правого сайдбара, удаляем все элементы с тегами: <label>, <p>, <input>, <select>
 *
 * @param rightSidebarBlock - объект блока правого сайдбара
 */
const clearRightSidebar = (rightSidebarBlock) => {
  // Удаляем все элементы с тегом <label>
  Array.prototype.slice
    .call(rightSidebarBlock.getElementsByTagName("label"))
    .forEach(function (item) {
      item.remove();
    });

  // Удаляем все элементы с тегом <p>
  Array.prototype.slice
    .call(rightSidebarBlock.getElementsByTagName("p"))
    .forEach(function (item) {
      item.remove();
    });

  // Удаляем все элементы с тегом <input>
  Array.prototype.slice
    .call(rightSidebarBlock.getElementsByTagName("input"))
    .forEach(function (item) {
      item.remove();
    });

  // Удаляем все элементы с тегом <select>
  Array.prototype.slice
    .call(rightSidebarBlock.getElementsByTagName("select"))
    .forEach(function (item) {
      item.remove();
    });
};

/**
 * Функция, реализующая удаление всех объектов с холста
 *
 */
const CanvasClearAll = () => {
  d3.select("svg").selectAll("*").remove(); // у тега <svg> выбираем всех его детей и удаляем
};

const DOMChangeColor = (element, color) => {
  if (element) {
    element.style.backgroundColor = color || null;
  }
};

const CanvasChangeBorderColor = (elementsID, color) => {
  elementsID.forEach((elementID) => {
    let element = svgCanvas.getElementById(elementID);
    console.log(element);
    element.setAttribute("stroke", color);
  });
};
