class Graph {
    constructor() {
        this.vertices = {}
    }

    addVertex(vertex, label) {
        this.vertices[vertex] = {}
        this.vertices[vertex]["label"] = label
        this.vertices[vertex]["edges"] = []
    }

    addEdge(vertex1, vertex2, label) {
        this.vertices[vertex1]["edges"].push({"label": label, "value": vertex2})
        this.vertices[vertex2]["edges"].push({"label": label, "value": vertex1})
    }

    removeEdge(vertex1, vertex2) {
        console.log(`______REMOVE______${vertex1}, ${vertex2}`)
        for (let i = 0; i < this.vertices[vertex1]["edges"].length; ++i) {
            if (this.vertices[vertex1]["edges"][i]["value"] === vertex2) {
                this.vertices[vertex1]["edges"].splice(i, 1)
            }
        }
        for (let i = 0; i < this.vertices[vertex2]["edges"].length; ++i) {
            if (this.vertices[vertex2]["edges"][i]["value"] === vertex1) {
                this.vertices[vertex2]["edges"].splice(i, 1)
            }
        }
    }

    removeVertex(vertex) {
        while (this.vertices[vertex]["edges"].length) {
            const vertex2 = this.vertices[vertex]["edges"].pop()
            this.removeEdge(vertex, vertex2["value"])

            this.debugIschemZhuka()
        }
        delete this.vertices[vertex]

        this.debugIschemZhuka()
    }

    debugIschemZhuka() {
        console.log('_____DEBUG_____')
        for (let vertex in this.vertices) {
            console.log(this.vertices[vertex])
        }
    }
}

(function test() {
    let graph = new Graph()

    graph.addVertex('Node1', 'label Node1')
    graph.addVertex('Node2', 'label Node2')
    graph.addVertex('Node3', 'label Node3')

    graph.addEdge('Node1', 'Node3', 'edge 1->3')
    graph.addEdge('Node1', 'Node2', 'edge 1->2')
    graph.addEdge('Node2', 'Node3', 'edge 2->3')

    graph.removeVertex('Node1')
})()
