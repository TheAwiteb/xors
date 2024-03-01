import { useSelector, useDispatch } from "react-redux";
import { Board, Players } from "../../Components";
import { getOneVsOne, play, newGame } from "../../app/slices/oneVsone";
import "./onevsone.css";

const OneVsOne = () => {
  const state = useSelector(getOneVsOne);
  const dispatch = useDispatch();

  return (
    <div className="onevsone">
      <div className="container">
        <Players players={state.players} round={state.round} />
        <Board
          board={state.board}
          combs={state.winner[1]}
          reducer={(index) => dispatch(play(index))}
        />
        {state.winner[0] && (
          <button onClick={() => dispatch(newGame())} alt="Play Again">
            العب مرة أخرى
          </button>
        )}
      </div>
    </div>
  );
};

export default OneVsOne;
