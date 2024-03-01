import "./board.css";
import PropTypes from "prop-types";

const XSymbol = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    className="x"
    viewBox="0 -960 960 960"
  >
    <path d="m256-200-56-56 224-224-224-224 56-56 224 224 224-224 56 56-224 224 224 224-56 56-224-224-224 224Z" />
  </svg>
);
const OSymbol = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    className="o"
    viewBox="0 -960 960 960"
  >
    <path d="M480-80q-83 0-156-31.5T197-197q-54-54-85.5-127T80-480q0-83 31.5-156T197-763q54-54 127-85.5T480-880q83 0 156 31.5T763-763q54 54 85.5 127T880-480q0 83-31.5 156T763-197q-54 54-127 85.5T480-80Zm0-80q134 0 227-93t93-227q0-134-93-227t-227-93q-134 0-227 93t-93 227q0 134 93 227t227 93Zm0-320Z" />
  </svg>
);

const Board = ({ board, combs, reducer }) => {
  const renderCell = (index) => {
    const symbol = board[index];
    return (
      <td
        key={index}
        className={combs.includes(index) ? "winner" : null}
        onClick={() => reducer(index)}
      >
        {symbol === "x" && <XSymbol />}
        {symbol === "o" && <OSymbol />}
      </td>
    );
  };

  return (
    <table className="board">
      <tbody>
        {[0, 1, 2].map((row) => (
          <tr key={row}>{[0, 1, 2].map((col) => renderCell(row * 3 + col))}</tr>
        ))}
      </tbody>
    </table>
  );

  // return (
  //   <table className="board">
  //     <tbody>
  //       <tr>
  //         <td onClick={() => dispatch(play(0))}>
  //           {board[0] === "x" ? (
  //             <XSymbol />
  //           ) : board[0] === "o" ? (
  //             <OSymbol />
  //           ) : null}
  //         </td>
  //         <td onClick={() => dispatch(play(1))}>
  //           {board[1] === "x" ? (
  //             <XSymbol />
  //           ) : board[1] === "o" ? (
  //             <OSymbol />
  //           ) : null}
  //         </td>
  //         <td onClick={() => dispatch(play(2))}>
  //           {board[2] === "x" ? (
  //             <XSymbol />
  //           ) : board[2] === "o" ? (
  //             <OSymbol />
  //           ) : null}
  //         </td>
  //       </tr>
  //       <tr>
  //         <td onClick={() => dispatch(play(3))}>
  //           {board[3] === "x" ? (
  //             <XSymbol />
  //           ) : board[3] === "o" ? (
  //             <OSymbol />
  //           ) : null}
  //         </td>
  //         <td onClick={() => dispatch(play(4))}>
  //           {board[4] === "x" ? (
  //             <XSymbol />
  //           ) : board[4] === "o" ? (
  //             <OSymbol />
  //           ) : null}
  //         </td>
  //         <td onClick={() => dispatch(play(5))}>
  //           {board[5] === "x" ? (
  //             <XSymbol />
  //           ) : board[5] === "o" ? (
  //             <OSymbol />
  //           ) : null}
  //         </td>
  //       </tr>
  //       <tr>
  //         <td onClick={() => dispatch(play(6))}>
  //           {board[6] === "x" ? (
  //             <XSymbol />
  //           ) : board[6] === "o" ? (
  //             <OSymbol />
  //           ) : null}
  //         </td>
  //         <td onClick={() => dispatch(play(7))}>
  //           {board[7] === "x" ? (
  //             <XSymbol />
  //           ) : board[7] === "o" ? (
  //             <OSymbol />
  //           ) : null}
  //         </td>
  //         <td onClick={() => dispatch(play(8))}>
  //           {board[8] === "x" ? (
  //             <XSymbol />
  //           ) : board[8] === "o" ? (
  //             <OSymbol />
  //           ) : null}
  //         </td>
  //       </tr>
  //     </tbody>
  //   </table>
  // );
};

export default Board;

Board.propTypes = {
  board: PropTypes.array,
  combs: PropTypes.array,
  reducer: PropTypes.func,
};
