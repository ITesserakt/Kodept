// 1) Списки смежности
// Для каждой вершины храним список вершин в которые можно попасть из нее

class Graph {
    constructor(size) {
        this.adjacencyList = new Array(size)

        for (let i = 0; i < this.adjacencyList.lengtf; i++) {
            this.adjacencyList[i] = new Array(0)
        }
    }

    AddEdge = (from, to) => {
        if (this.adjacencyList[from] === undefined) {
            this.adjacencyList[from] = []
        }
        this.adjacencyList[from].push(to)
    }

    VerticesCount = () => {
        return this.adjacencyList.length
    }

    GetNextVertices = (vertex) => {
        return this.adjacencyList[vertex]
    }

    GetPrevVertices = (vertex) => {
        let prevVertices = []

        for (let from = 0; from < this.adjacencyList.length; ++from) {
            this.adjacencyList[from].forEach(to => {
                if (to === vertex) {
                    prevVertices.push(from)
                }
            })
        }

        return prevVertices
    }
}

BFS = (graph, vertex, visited, f) => {
    let queue = []
    queue.push(vertex)

    visited[vertex] = true

    while (queue.length > 0) {
        let currentVertex = queue.shift()
        f(currentVertex)

        for (let nextVertex of graph.GetNextVertices(currentVertex)) {
            if (!visited[nextVertex]) {
                visited[nextVertex] = true
                queue.push(nextVertex)
            }
        }
    }
}

mainBFS = (graph, f) => {
    let visited = new Array(graph.VerticesCount())
    for (let i = 0; i < visited.length; ++i) {
        visited[i] = false;
    }

    for (let i = 0; i < graph.VerticesCount(); ++i) {
        if (!visited[i]) {
            BFS(graph, i, visited, f)
        }
    }
}

let cycle_st = 0
let cycle_end = 0
cycleDFS = (graph, vertex, visited, parents) => {
    visited[vertex] = 1
    for (let to of graph.GetNextVertices(vertex)) {
        if (visited[to] === 0) {
            parents[to] = vertex
            if (cycleDFS(graph, to, visited, parents)) {
                return true
            }
        } else if (visited[to] === 1) {
            cycle_st = tov
            cycle_end = vertex;
            return true;
        }
    }
    visited[vertex] = 2;
    return false;
}

mainCycle = (graph) => {
    for (let i = 0; i < graph.VerticesCount(); ++i) {
        let visited = new Array(graph.VerticesCount())
        for (let i = 0; i < visited.length; ++i) {
            visited[i] = 0;
        }
        let parents = new Array(graph.VerticesCount())

        if (cycleDFS(graph, i, visited, parents)) {
            let cycle = []
            cycle.push(cycle_st)
            for (let v = cycle_end; v !== cycle_st; v = parents[v]) {
                cycle.push(v)
            }
            cycle.push(cycle_st)
            cycle = cycle.reverse()
            console.log(cycle)

            cycle_st = 0
            cycle_end = 0
        }
    }
}

print = (vertex) => {
    console.log(vertex)
}

let graph = new Graph(6)
graph.AddEdge(0, 1)
graph.AddEdge(0, 5)
graph.AddEdge(1, 2)
graph.AddEdge(1, 3)
graph.AddEdge(1, 5)
graph.AddEdge(1, 6)
graph.AddEdge(5, 6)
graph.AddEdge(5, 4)
graph.AddEdge(6, 4)
graph.AddEdge(3, 6)
graph.AddEdge(3, 4)
graph.AddEdge(1, 0)

/*graph.AddEdge(0, 1)
graph.AddEdge(1, 0)*/

mainCycle(graph)

