import './style.css'

const canvas = document.getElementById('canvas') as HTMLCanvasElement;
const ctx = canvas.getContext('2d')!;

const startBtn = document.getElementById('start') as HTMLButtonElement;
const pauseBtn = document.getElementById('pause') as HTMLButtonElement;
const resetBtn = document.getElementById('reset') as HTMLButtonElement;

let animationId: number;
let isRunning = false;
let currentStep = 0;
let steps: AlgorithmStep[] = [];
let animationSpeed = 500; // ms

interface SortData {
  array: number[];
  i: number;
  j: number;
}

interface AlgorithmStep {
  step: number;
  data: SortData;
}

async function fetchBubbleSort(size: number = 10) {
  const response = await fetch('/api/sort', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      type: 'sort',
      params: {
        algorithm: 'bubble',
        size: size
      }
    })
  });
  return await response.json();
}

function drawArray(arr: number[], highlightI: number = -1, highlightJ: number = -1) {
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  const barWidth = canvas.width / arr.length;
  const maxVal = Math.max(...arr);

  arr.forEach((val, index) => {
    const barHeight = (val / maxVal) * (canvas.height - 40);
    const x = index * barWidth;
    const y = canvas.height - barHeight - 20;

    if (index === highlightI || index === highlightJ) {
      ctx.fillStyle = 'red';
    } else {
      ctx.fillStyle = 'blue';
    }

    ctx.fillRect(x, y, barWidth - 2, barHeight);

    ctx.fillStyle = 'black';
    ctx.font = '12px Arial';
    ctx.textAlign = 'center';
    ctx.fillText(val.toString(), x + barWidth / 2, y - 5);
  });
}

function animate() {
  if (isRunning && currentStep < steps.length) {
    const step = steps[currentStep];
    const data = step.data as SortData;
    drawArray(data.array, data.i, data.j);
    currentStep++;
    setTimeout(() => {
      animationId = requestAnimationFrame(animate);
    }, animationSpeed);
  } else {
    isRunning = false;
  }
}

startBtn.addEventListener('click', async () => {
  if (!isRunning) {
    steps = await fetchBubbleSort(10);
    currentStep = 0;
    isRunning = true;
    animate();
  }
});

pauseBtn.addEventListener('click', () => {
  isRunning = false;
  cancelAnimationFrame(animationId);
});

resetBtn.addEventListener('click', () => {
  isRunning = false;
  cancelAnimationFrame(animationId);
  currentStep = 0;
  if (steps.length > 0) {
    drawArray(steps[0].data.array);
  }
});

// Initial draw
drawArray([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);