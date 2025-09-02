package main

import (
	"math/rand"
)

func generateMaze(width, height int) [][]int {
	maze := make([][]int, height)
	for i := range maze {
		maze[i] = make([]int, width)
		for j := range maze[i] {
			maze[i][j] = 1 // wall
		}
	}

	// Start from top-left
	carve(maze, 1, 1)

	return maze
}

func carve(maze [][]int, x, y int) {
	directions := [][]int{{0, 1}, {1, 0}, {0, -1}, {-1, 0}}
	rand.Shuffle(len(directions), func(i, j int) {
		directions[i], directions[j] = directions[j], directions[i]
	})

	maze[y][x] = 0 // path

	for _, dir := range directions {
		nx, ny := x+dir[0]*2, y+dir[1]*2
		if nx > 0 && nx < len(maze[0])-1 && ny > 0 && ny < len(maze)-1 && maze[ny][nx] == 1 {
			maze[y+dir[1]][x+dir[0]] = 0
			carve(maze, nx, ny)
		}
	}
}
