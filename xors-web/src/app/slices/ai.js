import { createSlice } from "@reduxjs/toolkit";

const initialState = {
  players: [
    {
      name: "أنت",
      username: "",
      img: "",
      rank: null,
      points: 0,
      round: "x",
    },
    {
      name: "الذكاء الاصطناعي",
      username: "الخسارة اجباري والتعادل اختياري",
      img: "",
      rank: null,
      points: 0,
      round: "o",
    },
  ],
  board: ["", "", "", "", "", "", "", "", ""],
  round: "x",
  winner: ["", []],
};

const combs = [
  [0, 1, 2],
  [3, 4, 5],
  [6, 7, 8],
  [0, 3, 6],
  [1, 4, 7],
  [2, 5, 8],
  [0, 4, 8],
  [2, 4, 6],
];

function checkForWin(board) {
  let winner = ["", []];
  combs.map((e, i) => {
    board[e[0]] === board[e[1]] &&
    board[e[0]] === board[e[2]] &&
    board[e[0]] !== ""
      ? ((winner[0] = board[e[0]]), (winner[1] = combs[i]))
      : null;
  });
  return winner;
}

function checkForTie(board) {
  return board.every((e) => e !== "");
}

function avilable(board) {
  return board.reduce((acc, cur, index) => {
    if (cur === "") acc.push(index);
    return acc;
  }, []);
}

function result(board, action, player) {
  let newBoard = [...board];
  newBoard[action] = player;
  return newBoard;
}

function minimax(board, ismax) {
  const winner = checkForWin(board)[0];
  if (winner === "o") return 1;
  if (winner === "x") return -1;
  if (checkForTie(board)) return 0;

  if (ismax) {
    let bestScore = -Infinity;
    const av = avilable(board);
    av.forEach((spot) => {
      const newBoard = result(board, spot, "o");
      const score = minimax(newBoard, false);
      bestScore = Math.max(score, bestScore);
    });
    return bestScore;
  } else {
    let bestScore = Infinity;
    const av = avilable(board);
    av.forEach((spot) => {
      const newBoard = result(board, spot, "x");
      const score = minimax(newBoard, true);
      bestScore = Math.min(score, bestScore);
    });
    return bestScore;
  }
}

function getBestMove(board) {
  const av = avilable(board);
  let bestMove = -1;
  let bestVal = -Infinity;
  av.map((spot) => {
    const newBoard = result(board, spot, "o");
    const moveVal = minimax(newBoard, false);
    if (moveVal > bestVal) {
      bestVal = moveVal;
      bestMove = spot;
    }
  });
  return bestMove;
}

export const ai = createSlice({
  name: "ai",
  initialState,
  reducers: {
    play: (state, action) => {
      const index = action.payload;
      const board = state.board;
      if (!board[index] && !state.winner[0]) {
        board[index] = state.round;
        state.round = state.round === "x" ? "o" : "x";
        if (state.round === "o") {
          const bestMove = getBestMove(board);
          board[bestMove] = state.round;
          state.round = state.round === "x" ? "o" : "x";
        }
        const winner = checkForWin(board);
        if (winner[0]) {
          state.winner = winner;
          state.players.forEach((player) =>
            player.round === state.winner[0] ? (player.points += 1) : null
          );
        } else if (checkForTie(board)) {
          state.winner[0] = "Draw";
        }
      }
    },
    newGame: (state) => {
      state.board = ["", "", "", "", "", "", "", "", ""];
      state.winner = ["", []];
    },
  },
});

export const { play, newGame } = ai.actions;

export const selectBoard = (state) => state.ai;

export default ai.reducer;
