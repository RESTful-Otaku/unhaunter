package main

import (
	"encoding/json"
	"fmt"
	"net/http"
)

type AlgorithmRequest struct {
	Type   string                 `json:"type"`
	Params map[string]interface{} `json:"params"`
}

type AlgorithmStep struct {
	Step int         `json:"step"`
	Data interface{} `json:"data"`
}

type SortData struct {
	Array []int `json:"array"`
	I     int   `json:"i"`
	J     int   `json:"j"`
}

func main() {
	http.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		fmt.Fprintf(w, "Algo Visualizer Backend API")
	})

	http.HandleFunc("/api/sort", handleSort)
	http.HandleFunc("/api/search", handleSearch)
	http.HandleFunc("/api/pathfind", handlePathfind)

	fmt.Println("Server starting on :8080")
	http.ListenAndServe(":8080", nil)
}

func handleSort(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	var req AlgorithmRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	algorithm := req.Params["algorithm"].(string)
	size := int(req.Params["size"].(float64))

	var steps []AlgorithmStep

	switch algorithm {
	case "bubble":
		steps = generateBubbleSort(size)
	case "quick":
		steps = generateQuickSort(size)
	default:
		http.Error(w, "Unknown sort algorithm", http.StatusBadRequest)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(steps)
}

func handleSearch(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	var req AlgorithmRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	algorithm := req.Params["algorithm"].(string)
	size := int(req.Params["size"].(float64))
	target := int(req.Params["target"].(float64))

	var steps []AlgorithmStep

	switch algorithm {
	case "binary":
		steps = generateBinarySearch(size, target)
	default:
		http.Error(w, "Unknown search algorithm", http.StatusBadRequest)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(steps)
}

func handlePathfind(w http.ResponseWriter, r *http.Request) {
	// Placeholder for pathfinding algorithms
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode([]AlgorithmStep{{Step: 1, Data: "Pathfinding not implemented yet"}})
}

func generateBubbleSort(size int) []AlgorithmStep {
	arr := make([]int, size)
	for i := range arr {
		arr[i] = size - i
	}

	steps := []AlgorithmStep{}
	step := 0

	for i := 0; i < len(arr); i++ {
		for j := 0; j < len(arr)-1-i; j++ {
			step++
			steps = append(steps, AlgorithmStep{
				Step: step,
				Data: SortData{Array: append([]int(nil), arr...), I: i, J: j},
			})
			if arr[j] > arr[j+1] {
				arr[j], arr[j+1] = arr[j+1], arr[j]
			}
		}
	}

	return steps
}

func generateQuickSort(size int) []AlgorithmStep {
	arr := make([]int, size)
	for i := range arr {
		arr[i] = size - i
	}

	steps := []AlgorithmStep{}
	step := 0

	quickSort(arr, 0, len(arr)-1, &steps, &step)
	return steps
}

func quickSort(arr []int, low, high int, steps *[]AlgorithmStep, step *int) {
	if low < high {
		pivotIndex := partition(arr, low, high, steps, step)
		quickSort(arr, low, pivotIndex-1, steps, step)
		quickSort(arr, pivotIndex+1, high, steps, step)
	}
}

func partition(arr []int, low, high int, steps *[]AlgorithmStep, step *int) int {
	pivot := arr[high]
	i := low - 1

	for j := low; j < high; j++ {
		*step++
		*steps = append(*steps, AlgorithmStep{
			Step: *step,
			Data: SortData{Array: append([]int(nil), arr...), I: i, J: j},
		})
		if arr[j] < pivot {
			i++
			arr[i], arr[j] = arr[j], arr[i]
		}
	}
	arr[i+1], arr[high] = arr[high], arr[i+1]
	return i + 1
}
