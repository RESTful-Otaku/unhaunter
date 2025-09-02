interface Point {
  x: number;
  y: number;
}

class PathfindingVisualizer {
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private maze: number[][] = [];
  private start: Point | null = null;
  private end: Point | null = null;
  private path: Point[] = [];
  private cellSize = 20;
  private width = 20;
  private height = 20;

  constructor() {
    this.canvas = document.getElementById('canvas') as HTMLCanvasElement;
    this.ctx = this.canvas.getContext('2d')!;
    this.canvas.width = this.width * this.cellSize;
    this.canvas.height = this.height * this.cellSize;
    this.setupEventListeners();
    this.generateMaze();
  }

  private setupEventListeners() {
    this.canvas.addEventListener('click', this.handleCanvasClick.bind(this));
    document.getElementById('generate')!.addEventListener('click', this.generateMaze.bind(this));
    document.getElementById('run')!.addEventListener('click', this.runPathfinding.bind(this));
  }

  private async generateMaze() {
    const response = await fetch(`/maze?width=${this.width}&height=${this.height}`);
    this.maze = await response.json();
    this.start = null;
    this.end = null;
    this.path = [];
    this.draw();
  }

  private handleCanvasClick(event: MouseEvent) {
    const rect = this.canvas.getBoundingClientRect();
    const x = Math.floor((event.clientX - rect.left) / this.cellSize);
    const y = Math.floor((event.clientY - rect.top) / this.cellSize);

    if (!this.start) {
      this.start = { x, y };
    } else if (!this.end) {
      this.end = { x, y };
    } else {
      this.start = { x, y };
      this.end = null;
    }
    this.path = [];
    this.draw();
  }

  private async runPathfinding() {
    if (!this.start || !this.end) return;

    const algo = (document.getElementById('algo') as HTMLSelectElement).value;
    const response = await fetch('/path', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        maze: this.maze,
        start: this.start,
        end: this.end,
        algo
      })
    });
    const data = await response.json();
    this.path = data.path;
    this.draw();
  }

  private draw() {
    this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);

    // Draw maze
    for (let y = 0; y < this.height; y++) {
      for (let x = 0; x < this.width; x++) {
        if (this.maze[y][x] === 1) {
          this.ctx.fillStyle = '#000';
          this.ctx.fillRect(x * this.cellSize, y * this.cellSize, this.cellSize, this.cellSize);
        } else {
          this.ctx.fillStyle = '#fff';
          this.ctx.fillRect(x * this.cellSize, y * this.cellSize, this.cellSize, this.cellSize);
        }
      }
    }

    // Draw start
    if (this.start) {
      this.ctx.fillStyle = '#0f0';
      this.ctx.fillRect(this.start.x * this.cellSize, this.start.y * this.cellSize, this.cellSize, this.cellSize);
    }

    // Draw end
    if (this.end) {
      this.ctx.fillStyle = '#f00';
      this.ctx.fillRect(this.end.x * this.cellSize, this.end.y * this.cellSize, this.cellSize, this.cellSize);
    }

    // Draw path
    this.ctx.fillStyle = '#00f';
    for (const point of this.path) {
      this.ctx.fillRect(point.x * this.cellSize, point.y * this.cellSize, this.cellSize, this.cellSize);
    }
  }
}

new PathfindingVisualizer();