package main

import (
	"container/heap"
	"math"
)

type Item struct {
	point    Point
	priority int
	index    int
}

type PriorityQueue []*Item

func (pq PriorityQueue) Len() int { return len(pq) }

func (pq PriorityQueue) Less(i, j int) bool {
	return pq[i].priority < pq[j].priority
}

func (pq PriorityQueue) Swap(i, j int) {
	pq[i], pq[j] = pq[j], pq[i]
	pq[i].index = i
	pq[j].index = j
}

func (pq *PriorityQueue) Push(x any) {
	n := len(*pq)
	item := x.(*Item)
	item.index = n
	*pq = append(*pq, item)
}

func (pq *PriorityQueue) Pop() any {
	old := *pq
	n := len(old)
	item := old[n-1]
	old[n-1] = nil
	item.index = -1
	*pq = old[0 : n-1]
	return item
}

func heuristic(a, b Point) int {
	return int(math.Abs(float64(a.X-b.X)) + math.Abs(float64(a.Y-b.Y)))
}

func astar(maze [][]int, start, end Point) []Point {
	if maze[start.Y][start.X] == 1 || maze[end.Y][end.X] == 1 {
		return nil
	}

	openSet := &PriorityQueue{}
	heap.Init(openSet)
	heap.Push(openSet, &Item{point: start, priority: 0})

	cameFrom := make(map[Point]Point)
	gScore := make(map[Point]int)
	gScore[start] = 0
	fScore := make(map[Point]int)
	fScore[start] = heuristic(start, end)

	for openSet.Len() > 0 {
		current := heap.Pop(openSet).(*Item).point

		if current == end {
			return reconstructPath(cameFrom, current)
		}

		for _, dir := range [][]int{{0, 1}, {1, 0}, {0, -1}, {-1, 0}} {
			neighbour := Point{current.X + dir[0], current.Y + dir[1]}
			if neighbour.X < 0 || neighbour.X >= len(maze[0]) || neighbour.Y < 0 || neighbour.Y >= len(maze) || maze[neighbour.Y][neighbour.X] == 1 {
				continue
			}

			tentativeGScore := gScore[current] + 1
			if g, ok := gScore[neighbour]; !ok || tentativeGScore < g {
				cameFrom[neighbour] = current
				gScore[neighbour] = tentativeGScore
				fScore[neighbour] = tentativeGScore + heuristic(neighbour, end)
				heap.Push(openSet, &Item{point: neighbour, priority: fScore[neighbour]})
			}
		}
	}

	return nil
}

func bfs(maze [][]int, start, end Point) []Point {
	if maze[start.Y][start.X] == 1 || maze[end.Y][end.X] == 1 {
		return nil
	}

	queue := []Point{start}
	cameFrom := make(map[Point]Point)
	visited := make(map[Point]bool)
	visited[start] = true

	for len(queue) > 0 {
		current := queue[0]
		queue = queue[1:]

		if current == end {
			return reconstructPath(cameFrom, current)
		}

		for _, dir := range [][]int{{0, 1}, {1, 0}, {0, -1}, {-1, 0}} {
			neighbour := Point{current.X + dir[0], current.Y + dir[1]}
			if neighbour.X < 0 || neighbour.X >= len(maze[0]) || neighbour.Y < 0 || neighbour.Y >= len(maze) || maze[neighbour.Y][neighbour.X] == 1 || visited[neighbour] {
				continue
			}
			visited[neighbour] = true
			queue = append(queue, neighbour)
			cameFrom[neighbour] = current
		}
	}

	return nil
}

func reconstructPath(cameFrom map[Point]Point, current Point) []Point {
	path := []Point{current}
	for {
		if prev, ok := cameFrom[current]; ok {
			current = prev
			path = append([]Point{current}, path...)
		} else {
			break
		}
	}
	return path
}
