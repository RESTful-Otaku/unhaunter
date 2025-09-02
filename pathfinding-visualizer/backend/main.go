package main

import (
	"encoding/json"
	"log"
	"net/http"
	"strconv"
)

type Point struct {
	X, Y int
}

type MazeRequest struct {
	Width, Height int
}

type PathRequest struct {
	Maze  [][]int `json:"maze"`
	Start Point   `json:"start"`
	End   Point   `json:"end"`
	Algo  string  `json:"algo"` // "astar" or "bfs"
}

type PathResponse struct {
	Path []Point `json:"path"`
}

func main() {
	http.HandleFunc("/maze", handleMaze)
	http.HandleFunc("/path", handlePath)
	http.Handle("/", http.FileServer(http.Dir("../frontend")))

	log.Println("Server starting on :8080")
	log.Fatal(http.ListenAndServe(":8080", nil))
}

func handleMaze(w http.ResponseWriter, r *http.Request) {
	widthStr := r.URL.Query().Get("width")
	heightStr := r.URL.Query().Get("height")
	width, _ := strconv.Atoi(widthStr)
	height, _ := strconv.Atoi(heightStr)
	if width == 0 {
		width = 20
	}
	if height == 0 {
		height = 20
	}

	maze := generateMaze(width, height)
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(maze)
}

func handlePath(w http.ResponseWriter, r *http.Request) {
	var req PathRequest
	json.NewDecoder(r.Body).Decode(&req)

	var path []Point
	if req.Algo == "astar" {
		path = astar(req.Maze, req.Start, req.End)
	} else {
		path = bfs(req.Maze, req.Start, req.End)
	}

	resp := PathResponse{Path: path}
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(resp)
}
