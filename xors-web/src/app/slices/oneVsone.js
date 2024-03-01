import { createSlice } from "@reduxjs/toolkit";

const initialState = {
  players: [
    {
      name: "اللاعب 1",
      username: "",
      img: "",
      rank: null,
      points: 0,
      round: "x",
    },
    {
      name: "اللاعب 2",
      username: "",
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

export const oneVsOne = createSlice({
  name: "onevsone",
  initialState,
  reducers: {
    play: (state, action) => {
      const index = action.payload;
      if (!state.board[index] && !state.winner[0]) {
        state.board[index] = state.round;
        state.round = state.round === "x" ? "o" : "x";
        const winner = checkForWin(state.board);
        if (winner[0]) {
          state.winner = winner;
          state.players.forEach((player) =>
            player.round === state.winner[0] ? (player.points += 1) : null
          );
        } else if (state.board.every((e) => e !== "")) {
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

export const { play, newGame } = oneVsOne.actions;

export const getOneVsOne = (state) => state.oneVsOne;

export default oneVsOne.reducer;
