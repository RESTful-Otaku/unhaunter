# Pathfinding Visualizer

A mini project visualizing pathfinding algorithms on a procedurally generated maze.

## Features

- Procedurally generated maze using recursive backtracking
- Pathfinding algorithms: A* and BFS
- Interactive UI to set start and end points
- Regenerate maze
- Minimal dependencies

## Tech Stack

- Backend: Go (standard library only)
- Frontend: TypeScript (compiled to JavaScript)

## Setup

1. Install Go and Node.js
2. Clone or download the project
3. Backend: `cd backend && go run main.go`
4. Frontend: `cd frontend && npm install && npm run build`
5. Open browser to http://localhost:8080

## Usage

- Click "Generate Maze" to create a new maze
- Click on the grid to set start (green) and end (red) points
- Select algorithm (A* or BFS)
- Click "Run Pathfinding" to visualize the path (blue)