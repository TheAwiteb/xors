import "./players.css";
import PropTypes from "prop-types";

const Players = ({ players, round }) => {
  return (
    <div className="players">
      {players.map((e, i) => {
        return (
          <div
            className={e.round === round ? "current player" : "player"}
            key={i}
          >
            <div className="u">
              {e.rank && <h2 className="rank">{e.rank}</h2>}
              <img src={e.img ? e.img : "assets/user.svg"} alt={e.name} />
              <div className="info">
                <p className="name">{e.name}</p>
                {e.username && <p className="username">@{e.username}</p>}
              </div>
            </div>
            <h3>{e.points}</h3>
          </div>
        );
      })}
    </div>
  );
};

export default Players;

Players.propTypes = {
  round: PropTypes.string,
  players: PropTypes.array,
};
