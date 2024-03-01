import { useSelector, useDispatch } from "react-redux";
import { Board, Players } from "../../Components";
import { selectBoard, play, newGame } from "../../app/slices/ai";
import "./ai.css";

const TicTacToe = () => {
  const state = useSelector(selectBoard);
  const dispatch = useDispatch();

  return (
    <div className="ai">
      <div className="container">
        <Players players={state.players} round={state.round} />
        <Board
          board={state.board}
          combs={state.winner[1]}
          reducer={(index) => dispatch(play(index))}
        />
        {state.winner[0] && (
          <button onClick={() => dispatch(newGame())}>العب مرة أخرى</button>
        )}
      </div>
    </div>
  );
};

export default TicTacToe;
